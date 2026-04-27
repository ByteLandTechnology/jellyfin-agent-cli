//! Interactive REPL mode with readline, history, and tab completion.

use clap::{Command, CommandFactory};
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{CompletionType, Config, Context, EditMode, Editor, Helper};
use std::borrow::Cow;
use std::ffi::OsString;

use crate::output::{CommandOutput, OutputFormat};
use crate::{
    Cli, Commands, detect_requested_output_format, execute_command, handle_parse_error,
    render_structured_help_command, resolve_output_format, write_rendered,
};

/// History file name stored under the config directory.
const HISTORY_FILE: &str = "repl_history";

/// Maximum number of history entries to keep.
const HISTORY_MAX: usize = 1000;

// ---------------------------------------------------------------------------
// Tab completer
// ---------------------------------------------------------------------------

/// A completer that walks the clap `Command` tree to offer subcommand and flag
/// completions.
struct ClapCompleter {
    root: Command,
}

impl ClapCompleter {
    fn new() -> Self {
        Self {
            root: Cli::command(),
        }
    }

    /// Walk the already-typed tokens to find the deepest matching subcommand,
    /// then complete the partial last token against that subcommand's children
    /// and flags.
    fn complete_line(&self, line: &str, pos: usize) -> Vec<Pair> {
        let truncated = &line[..pos];
        let tokens = shell_split(truncated);
        let trailing_space = truncated.ends_with(' ') || truncated.is_empty();

        // Walk tokens to find the deepest subcommand.
        let mut current = &self.root;
        let mut consumed = 0;

        for token in &tokens {
            if token.starts_with('-') {
                // flags are not subcommands — stop descending
                consumed += 1;
                continue;
            }
            if let Some(sub) = current.find_subcommand(token) {
                current = sub;
                consumed += 1;
            } else {
                break;
            }
        }

        // Determine the partial word we are completing.
        let partial = if trailing_space {
            ""
        } else {
            tokens.last().map(|s| s.as_str()).unwrap_or("")
        };

        let mut candidates: Vec<Pair> = Vec::new();

        // If partial starts with '-', complete flags.
        if partial.starts_with('-') {
            for arg in current.get_arguments() {
                if let Some(long) = arg.get_long() {
                    let flag = format!("--{long}");
                    if flag.starts_with(partial) {
                        candidates.push(Pair {
                            display: flag.clone(),
                            replacement: flag,
                        });
                    }
                }
                if let Some(short) = arg.get_short() {
                    let flag = format!("-{short}");
                    if flag.starts_with(partial) {
                        candidates.push(Pair {
                            display: flag.clone(),
                            replacement: flag,
                        });
                    }
                }
            }
            // Also complete global args from root.
            if !std::ptr::eq(current, &self.root) {
                for arg in self.root.get_arguments() {
                    if let Some(long) = arg.get_long() {
                        let flag = format!("--{long}");
                        if flag.starts_with(partial)
                            && !candidates.iter().any(|c| c.replacement == flag)
                        {
                            candidates.push(Pair {
                                display: flag.clone(),
                                replacement: flag,
                            });
                        }
                    }
                }
            }
        } else {
            // Complete subcommands.
            for sub in current.get_subcommands() {
                let name = sub.get_name();
                if name == "help" {
                    continue;
                }
                if name.starts_with(partial) {
                    let about = sub.get_about().map(|a| a.to_string()).unwrap_or_default();
                    candidates.push(Pair {
                        display: if about.is_empty() {
                            name.to_string()
                        } else {
                            format!("{name:<16} {about}")
                        },
                        replacement: name.to_string(),
                    });
                }
            }

            // Also offer built-in REPL commands.
            if consumed == 0 {
                for builtin in ["exit", "quit", "help"] {
                    if builtin.starts_with(partial)
                        && !candidates.iter().any(|c| c.replacement == builtin)
                    {
                        candidates.push(Pair {
                            display: builtin.to_string(),
                            replacement: builtin.to_string(),
                        });
                    }
                }
            }
        }

        candidates.sort_by(|a, b| a.replacement.cmp(&b.replacement));
        candidates
    }
}

impl Completer for ClapCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        // Find where the partial token starts.
        let start = line[..pos]
            .rfind(|c: char| c.is_whitespace())
            .map(|i| i + 1)
            .unwrap_or(0);
        Ok((start, self.complete_line(line, pos)))
    }
}

impl Hinter for ClapCompleter {
    type Hint = String;
}

impl Highlighter for ClapCompleter {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> Cow<'b, str> {
        Cow::Owned(format!("\x1b[1;36m{prompt}\x1b[0m"))
    }
}

impl Validator for ClapCompleter {}

impl Helper for ClapCompleter {}

// ---------------------------------------------------------------------------
// REPL entry point
// ---------------------------------------------------------------------------

pub async fn run(profile: Option<String>, output_flag: Option<String>, debug: bool) -> i32 {
    crate::init_tracing(debug);

    let history_path = history_file_path();

    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .max_history_size(HISTORY_MAX)
        .expect("valid history size")
        .build();

    let mut rl = Editor::with_config(config).expect("failed to create editor");
    rl.set_helper(Some(ClapCompleter::new()));

    if let Some(ref path) = history_path {
        let _ = rl.load_history(path);
    }

    println!("Jellyfin CLI interactive mode. Type 'help' for commands, 'exit' to quit.");

    let exit_code;

    loop {
        let prompt = build_prompt(&profile);
        match rl.readline(&prompt) {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                let _ = rl.add_history_entry(trimmed);

                match trimmed {
                    "exit" | "quit" => {
                        exit_code = 0;
                        break;
                    }
                    "help" => {
                        print_repl_help();
                        continue;
                    }
                    _ => {}
                }

                // Parse the line into tokens.
                let tokens = match shellwords::split(trimmed) {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("Parse error: {e}");
                        continue;
                    }
                };

                // Build a full arg vector: prepend "jellyfin-agent-cli" + any persistent flags.
                let mut args: Vec<String> = vec!["jellyfin-agent-cli".to_string()];
                if let Some(ref fmt) = output_flag {
                    args.push("--output".to_string());
                    args.push(fmt.clone());
                }
                if let Some(ref p) = profile {
                    args.push("--profile".to_string());
                    args.push(p.clone());
                }
                if debug {
                    args.push("--debug".to_string());
                }
                args.extend(tokens);

                // Try to parse with clap.
                let raw_args: Vec<OsString> = args.iter().cloned().map(OsString::from).collect();
                let requested_output = detect_requested_output_format(&raw_args);
                let explicit_output_requested = requested_output.is_some();
                let mut command = Cli::command();
                let matches = match command.try_get_matches_from_mut(&raw_args) {
                    Ok(m) => m,
                    Err(e) => {
                        let _ = handle_parse_error(
                            &command,
                            &raw_args,
                            requested_output.unwrap_or(OutputFormat::Table),
                            &e,
                        );
                        continue;
                    }
                };

                let cli = match <Cli as clap::FromArgMatches>::from_arg_matches(&matches) {
                    Ok(cli) => cli,
                    Err(e) => {
                        eprintln!("{e}");
                        continue;
                    }
                };

                if let Commands::Help {
                    command_path,
                    format,
                } = &cli.command
                {
                    let _ = render_structured_help_command(&command, command_path, format);
                    continue;
                }

                let output_format = if explicit_output_requested {
                    match resolve_output_format(&cli) {
                        Ok(f) => f,
                        Err(e) => {
                            eprintln!("Output format error: {e}");
                            continue;
                        }
                    }
                } else {
                    OutputFormat::Table
                };

                match execute_command(cli).await {
                    Ok(envelope) => {
                        let text = crate::output::format_output(&envelope, output_format);
                        write_rendered(&text, false);
                    }
                    Err(error) => {
                        let command_path = "jellyfin-agent-cli";
                        let envelope = CommandOutput::from_command_error(command_path, &error);
                        let text = crate::output::format_output(&envelope, output_format);
                        write_rendered(&text, true);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl-C: cancel current line, don't exit.
                continue;
            }
            Err(ReadlineError::Eof) => {
                // Ctrl-D: exit.
                exit_code = 0;
                break;
            }
            Err(err) => {
                eprintln!("Readline error: {err}");
                exit_code = 1;
                break;
            }
        }
    }

    if let Some(ref path) = history_path {
        if let Err(e) = rl.save_history(path) {
            eprintln!("Warning: could not save history: {e}");
        }
    }

    exit_code
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_prompt(profile: &Option<String>) -> String {
    match profile {
        Some(p) => format!("jellyfin-agent-cli({p})> "),
        None => "jellyfin-agent-cli> ".to_string(),
    }
}

fn history_file_path() -> Option<std::path::PathBuf> {
    dirs::data_local_dir().map(|d| {
        let dir = d.join("jellyfin-cli");
        let _ = std::fs::create_dir_all(&dir);
        dir.join(HISTORY_FILE)
    })
}

fn print_repl_help() {
    println!(
        "\
Interactive commands:
  help          Show this help
  exit, quit    Exit the REPL (or press Ctrl-D)

All regular jellyfin-agent-cli commands are available without the 'jellyfin-agent-cli' prefix.
Tab completion is available for commands and flags.
Use Up/Down arrows to navigate command history."
    );
}

/// Split a string into shell-like tokens, handling incomplete quotes gracefully.
fn shell_split(input: &str) -> Vec<String> {
    shellwords::split(input).unwrap_or_else(|_| {
        // Fallback: simple whitespace split.
        input.split_whitespace().map(String::from).collect()
    })
}
