//! Help document structure and generators.

use clap::{Arg, ArgAction, Command};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::output::formatter::OutputFormat;
use jellyfin_core::{E2EEnvironment, MediaCache};

/// Structured help document for a command.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HelpDocument {
    /// The command this help is for.
    pub command: String,

    /// Brief summary of what the command does.
    pub summary: String,

    /// Usage syntax.
    pub usage: String,

    /// Available subcommands.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub subcommands: Vec<SubcommandInfo>,

    /// Positional arguments.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub arguments: Vec<ArgumentInfo>,

    /// Available options and flags.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<OptionInfo>,

    /// Supported result and help output formats.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub output_formats: Vec<FormatInfo>,

    /// Documented runtime directories and files.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub runtime_directories: Vec<RuntimeDirectoryInfo>,

    /// Active Context contract for this command.
    pub active_context: ActiveContextHelp,

    /// Notes or prerequisites that are useful before running the command.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub before_you_run_it: Vec<String>,

    /// Example usage.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<Example>,

    /// Related commands.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub see_also: Vec<RelatedCommand>,

    /// Suggested commands to try next.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub try_next: Vec<RelatedCommand>,

    /// Exit code reference for this command.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub exit_codes: Vec<ExitCodeInfo>,
}

impl HelpDocument {
    /// Create a new help document.
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            summary: String::new(),
            usage: String::new(),
            subcommands: Vec::new(),
            arguments: Vec::new(),
            options: Vec::new(),
            output_formats: Vec::new(),
            runtime_directories: Vec::new(),
            active_context: ActiveContextHelp::default(),
            before_you_run_it: Vec::new(),
            examples: Vec::new(),
            see_also: Vec::new(),
            try_next: Vec::new(),
            exit_codes: Vec::new(),
        }
    }

    /// Build a help document from clap reflection metadata.
    pub fn from_clap(command: &Command, command_path: impl Into<String>) -> Self {
        let command_path = command_path.into();
        let mut usage_command = command.clone();
        usage_command.set_bin_name(command_path.clone());
        let usage = usage_command.render_usage().to_string();

        let mut help = Self::new(command_path.clone())
            .with_summary(command_summary(command))
            .with_usage(usage)
            .with_subcommands(collect_subcommands(command, &command_path))
            .with_arguments(collect_arguments(command))
            .with_options(collect_options(command))
            .with_output_formats(default_output_formats())
            .with_runtime_directories(default_runtime_directories())
            .with_active_context(default_active_context())
            .with_before_you_run_it(vec![
                format!(
                    "Default structured format: {}.",
                    OutputFormat::default_structured()
                ),
                format!(
                    "Supported formats: {}.",
                    OutputFormat::supported_values().join(", ")
                ),
            ])
            .with_examples(default_examples(command, &command_path))
            .with_exit_codes(default_exit_codes())
            .with_try_next(default_try_next(command, &command_path));

        if let Some(parent_command) = parent_command(&command_path) {
            help = help.with_see_also(vec![RelatedCommand::new(
                format!("{parent_command} --help"),
                "Inspect the parent command.",
            )]);
        }

        help
    }

    /// Set the summary.
    pub fn with_summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = summary.into();
        self
    }

    /// Set the usage.
    pub fn with_usage(mut self, usage: impl Into<String>) -> Self {
        self.usage = usage.into();
        self
    }

    /// Replace subcommands.
    pub fn with_subcommands(mut self, infos: Vec<SubcommandInfo>) -> Self {
        self.subcommands = infos;
        self
    }

    /// Replace arguments.
    pub fn with_arguments(mut self, infos: Vec<ArgumentInfo>) -> Self {
        self.arguments = infos;
        self
    }

    /// Replace options.
    pub fn with_options(mut self, infos: Vec<OptionInfo>) -> Self {
        self.options = infos;
        self
    }

    /// Replace output format information.
    pub fn with_output_formats(mut self, infos: Vec<FormatInfo>) -> Self {
        self.output_formats = infos;
        self
    }

    /// Replace documented runtime directories.
    pub fn with_runtime_directories(mut self, infos: Vec<RuntimeDirectoryInfo>) -> Self {
        self.runtime_directories = infos;
        self
    }

    /// Replace Active Context help content.
    pub fn with_active_context(mut self, info: ActiveContextHelp) -> Self {
        self.active_context = info;
        self
    }

    /// Replace notes shown before execution.
    pub fn with_before_you_run_it(mut self, notes: Vec<String>) -> Self {
        self.before_you_run_it = notes;
        self
    }

    /// Append a note shown before execution.
    pub fn push_before_you_run_it(mut self, note: impl Into<String>) -> Self {
        self.before_you_run_it.push(note.into());
        self
    }

    /// Replace related commands.
    pub fn with_see_also(mut self, commands: Vec<RelatedCommand>) -> Self {
        self.see_also = commands;
        self
    }

    /// Replace suggested next commands.
    pub fn with_try_next(mut self, commands: Vec<RelatedCommand>) -> Self {
        self.try_next = commands;
        self
    }

    /// Replace example usage.
    pub fn with_examples(mut self, examples: Vec<Example>) -> Self {
        self.examples = examples;
        self
    }

    /// Replace exit code documentation.
    pub fn with_exit_codes(mut self, exit_codes: Vec<ExitCodeInfo>) -> Self {
        self.exit_codes = exit_codes;
        self
    }
}

/// Information about a subcommand.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubcommandInfo {
    /// Subcommand name.
    pub name: String,
    /// Brief description.
    pub summary: String,
}

impl SubcommandInfo {
    /// Create a new subcommand info.
    pub fn new(name: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            summary: summary.into(),
        }
    }
}

/// Information about a positional argument.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArgumentInfo {
    /// Argument name or placeholder.
    pub name: String,
    /// Description.
    pub description: String,
    /// Whether the argument is required.
    pub required: bool,
}

impl ArgumentInfo {
    /// Create a new argument info.
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            required: true,
        }
    }

    /// Mark as optional.
    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }
}

/// Information about an option or flag.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OptionInfo {
    /// The long flag, or the short flag when there is no long form.
    pub flag: String,
    /// Short flag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short: Option<String>,
    /// Description.
    pub summary: String,
    /// Default value if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    /// Possible values if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub possible: Option<Vec<String>>,
}

/// Structured help format selector for `help --format`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructuredHelpFormat {
    /// YAML help output.
    Yaml,
    /// JSON help output.
    Json,
    /// TOML help output.
    Toml,
}

impl StructuredHelpFormat {
    /// Parse a structured help format.
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "yaml" => Some(Self::Yaml),
            "json" => Some(Self::Json),
            "toml" => Some(Self::Toml),
            _ => None,
        }
    }

    /// Convert into the shared output format enum.
    pub fn into_output_format(self) -> OutputFormat {
        match self {
            Self::Yaml => OutputFormat::Yaml,
            Self::Json => OutputFormat::Json,
            Self::Toml => OutputFormat::Toml,
        }
    }
}

/// Output format reference.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FormatInfo {
    /// Format name.
    pub name: String,
    /// Where the format is supported.
    pub surface: String,
    /// Description of how to request it.
    pub summary: String,
    /// Whether it is the default on that surface.
    pub default: bool,
}

impl FormatInfo {
    /// Create a new format info record.
    pub fn new(
        name: impl Into<String>,
        surface: impl Into<String>,
        summary: impl Into<String>,
        default: bool,
    ) -> Self {
        Self {
            name: name.into(),
            surface: surface.into(),
            summary: summary.into(),
            default,
        }
    }
}

/// Runtime directory reference.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuntimeDirectoryInfo {
    /// Directory kind (config/data/state/cache/log).
    pub kind: String,
    /// Default path or file location.
    pub path: String,
    /// Scope or lifecycle note.
    pub scope: String,
    /// Override or discovery note.
    pub override_hint: String,
}

impl RuntimeDirectoryInfo {
    /// Create a new runtime directory reference.
    pub fn new(
        kind: impl Into<String>,
        path: impl Into<String>,
        scope: impl Into<String>,
        override_hint: impl Into<String>,
    ) -> Self {
        Self {
            kind: kind.into(),
            path: path.into(),
            scope: scope.into(),
            override_hint: override_hint.into(),
        }
    }
}

/// Active Context help section.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ActiveContextHelp {
    /// Inspect command.
    pub inspect_command: String,
    /// Persist command.
    pub persist_command: String,
    /// Per-invocation override flag.
    pub override_flag: String,
    /// Precedence rule.
    pub precedence: String,
    /// Summary of where the effective profile is surfaced.
    pub visibility: String,
}

/// Exit code reference.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExitCodeInfo {
    /// Numeric exit code.
    pub code: i32,
    /// What the code means.
    pub summary: String,
}

impl ExitCodeInfo {
    /// Create a new exit code record.
    pub fn new(code: i32, summary: impl Into<String>) -> Self {
        Self {
            code,
            summary: summary.into(),
        }
    }
}

impl OptionInfo {
    /// Create a new option info.
    pub fn new(flag: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            flag: flag.into(),
            short: None,
            summary: summary.into(),
            default: None,
            possible: None,
        }
    }

    /// Set the short flag.
    pub fn with_short(mut self, short: impl Into<String>) -> Self {
        self.short = Some(short.into());
        self
    }

    /// Set the default value.
    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }

    /// Set possible values.
    pub fn with_possible(mut self, values: Vec<String>) -> Self {
        self.possible = Some(values);
        self
    }
}

/// Example usage.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Example {
    /// Description of what the example does.
    pub description: String,
    /// The actual command.
    pub command: String,
}

/// Related or suggested command entry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelatedCommand {
    /// The command to run.
    pub command: String,
    /// Why this command is relevant.
    pub summary: String,
}

impl RelatedCommand {
    /// Create a new related command entry.
    pub fn new(command: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            summary: summary.into(),
        }
    }
}

/// Generate formatted human-readable help text from a help document.
pub fn format_help_human(help: &HelpDocument) -> String {
    let mut output = String::new();

    output.push_str("NAME\n");
    output.push_str(&format!("  {} - {}\n", help.command, help.summary));

    output.push_str("\nSYNOPSIS\n");
    output.push_str(&format!("  {}\n", help.usage));

    output.push_str("\nDESCRIPTION\n");
    output.push_str(&format!("  {}\n", help.summary));

    if !help.before_you_run_it.is_empty() {
        output.push_str("\n  Before you run it:\n");
        for note in &help.before_you_run_it {
            output.push_str(&format!("    - {}\n", note));
        }
    }

    if !help.subcommands.is_empty() {
        output.push_str("\n  Subcommands:\n");
        for sub in &help.subcommands {
            output.push_str(&format!("    {:<28} {}\n", sub.name, sub.summary));
        }
    }

    if !help.arguments.is_empty() {
        output.push_str("\n  Arguments:\n");
        for arg in &help.arguments {
            let required = if arg.required { "required" } else { "optional" };
            output.push_str(&format!(
                "    {:<28} {} [{}]\n",
                arg.name, arg.description, required
            ));
        }
    }

    if !help.runtime_directories.is_empty() {
        output.push_str("\n  Runtime directories:\n");
        for entry in &help.runtime_directories {
            output.push_str(&format!(
                "    {:<8} {} ({})\n",
                entry.kind, entry.path, entry.scope
            ));
        }
    }

    output.push_str("\nOPTIONS\n");
    if help.options.is_empty() {
        output.push_str("  This command has no options.\n");
    } else {
        for opt in &help.options {
            let flag = format_option_flags(opt);
            let mut metadata = Vec::new();

            if let Some(default) = &opt.default {
                metadata.push(format!("default: {default}"));
            }

            if let Some(possible) = &opt.possible {
                metadata.push(format!("possible: {}", possible.join(", ")));
            }

            let summary = if metadata.is_empty() {
                opt.summary.clone()
            } else {
                format!("{} [{}]", opt.summary, metadata.join("; "))
            };

            output.push_str(&format!("  {:<34} {}\n", flag, summary));
        }
    }

    output.push_str("\nFORMATS\n");
    for format in &help.output_formats {
        let default = if format.default { " [default]" } else { "" };
        output.push_str(&format!(
            "  {:<8} {:<18} {}{}\n",
            format.name, format.surface, format.summary, default
        ));
    }
    output.push_str(&format!(
        "  active-context {:<9} inspect: {}; persist: {}; override: {}; precedence: {}\n",
        "",
        help.active_context.inspect_command,
        help.active_context.persist_command,
        help.active_context.override_flag,
        help.active_context.precedence
    ));

    output.push_str("\nEXAMPLES\n");
    for example in &help.examples {
        output.push_str(&format!(
            "  # {}\n  {}\n",
            example.description, example.command
        ));
    }

    output.push_str("\nEXIT CODES\n");
    for exit_code in &help.exit_codes {
        output.push_str(&format!("  {:<3} {}\n", exit_code.code, exit_code.summary));
    }

    output
}

/// Backward-compatible alias used by older formatter paths.
pub fn format_help_table(help: &HelpDocument) -> String {
    format_help_human(help)
}

fn default_output_formats() -> Vec<FormatInfo> {
    vec![
        FormatInfo::new(
            "human",
            "--help / bare non-leaf invocation",
            "Human-readable man-like help output.",
            true,
        ),
        FormatInfo::new(
            "yaml",
            "help --format",
            "Structured help document in YAML.",
            true,
        ),
        FormatInfo::new(
            "json",
            "help --format",
            "Structured help document in JSON.",
            false,
        ),
        FormatInfo::new(
            "toml",
            "help --format",
            "Structured help document in TOML.",
            false,
        ),
        FormatInfo::new(
            "table/yaml/toml/json/ndjson",
            "--output",
            "Structured command results use the global --output flag.",
            false,
        ),
    ]
}

fn default_runtime_directories() -> Vec<RuntimeDirectoryInfo> {
    let config_root = user_scoped_path(dirs::config_dir(), &["jellyfin-cli"]);
    let data_root = user_scoped_path(dirs::data_local_dir(), &["jellyfin-cli"]);
    let e2e_state = E2EEnvironment::default_state_dir();
    let e2e_log = E2EEnvironment::default_log_dir();
    let e2e_cache = MediaCache::default_cache_dir();

    vec![
        RuntimeDirectoryInfo::new(
            "config",
            config_root.join("config.toml").display().to_string(),
            "user-scoped default",
            "Overridden by editing the config file or using explicit flags such as --server / --profile for one invocation.",
        ),
        RuntimeDirectoryInfo::new(
            "data",
            data_root.join("repl_history").display().to_string(),
            "user-scoped default",
            "REPL history is stored here automatically.",
        ),
        RuntimeDirectoryInfo::new(
            "state",
            e2e_state.join("server.pid").display().to_string(),
            "user-scoped default",
            "Changed indirectly with 'e2e setup --data-dir'.",
        ),
        RuntimeDirectoryInfo::new(
            "cache",
            e2e_cache.display().to_string(),
            "user-scoped default",
            "Changed with 'e2e media download --cache-dir'.",
        ),
        RuntimeDirectoryInfo::new(
            "log",
            e2e_log.display().to_string(),
            "user-scoped default",
            "Changed indirectly with 'e2e setup --data-dir'.",
        ),
    ]
}

fn default_active_context() -> ActiveContextHelp {
    ActiveContextHelp {
        inspect_command: "jellyfin-agent-cli context show".to_string(),
        persist_command: "jellyfin-agent-cli context use <NAME>".to_string(),
        override_flag: "--profile <NAME>".to_string(),
        precedence: "explicit --profile override wins for that invocation and never mutates persisted context".to_string(),
        visibility: "Command results include active_context metadata when context resolution matters.".to_string(),
    }
}

fn default_examples(command: &Command, command_path: &str) -> Vec<Example> {
    let mut examples = vec![
        Example {
            description: "Show human-readable help".to_string(),
            command: format!("{command_path} --help"),
        },
        Example {
            description: "Show structured help in YAML".to_string(),
            command: format!(
                "jellyfin-agent-cli help {}--format yaml",
                example_path_suffix(command_path)
            ),
        },
    ];

    if command.get_subcommands().next().is_some() {
        examples.push(Example {
            description: "Inspect the first child command".to_string(),
            command: format!(
                "{command_path} {}",
                command
                    .get_subcommands()
                    .next()
                    .map(|sub| sub.get_name())
                    .unwrap_or("")
            ),
        });
    }

    examples
}

fn default_exit_codes() -> Vec<ExitCodeInfo> {
    vec![
        ExitCodeInfo::new(
            0,
            "Success, human-readable help, or structured help rendered.",
        ),
        ExitCodeInfo::new(10, "Authentication error."),
        ExitCodeInfo::new(20, "Network error."),
        ExitCodeInfo::new(30, "API error."),
        ExitCodeInfo::new(
            40,
            "Input validation error, including missing required leaf input.",
        ),
        ExitCodeInfo::new(50, "Internal error."),
    ]
}

fn user_scoped_path(base: Option<PathBuf>, suffix: &[&str]) -> PathBuf {
    let mut path = base.unwrap_or_else(|| PathBuf::from("<user-dir>"));
    for segment in suffix {
        path = path.join(segment);
    }
    path
}

fn example_path_suffix(command_path: &str) -> String {
    if command_path == "jellyfin-agent-cli" {
        String::new()
    } else {
        format!(
            "{} ",
            command_path
                .strip_prefix("jellyfin-agent-cli ")
                .unwrap_or(command_path)
        )
    }
}

fn collect_subcommands(command: &Command, command_path: &str) -> Vec<SubcommandInfo> {
    let mut subcommands: Vec<_> = command
        .get_subcommands()
        .map(|sub| {
            let name = format!("{command_path} {}", sub.get_name());
            SubcommandInfo::new(name, command_summary(sub))
        })
        .collect();
    subcommands.sort_by(|left, right| left.name.cmp(&right.name));
    subcommands
}

fn collect_arguments(command: &Command) -> Vec<ArgumentInfo> {
    let mut arguments: Vec<_> = command
        .get_positionals()
        .filter(|arg| !arg.is_hide_set())
        .map(argument_from_clap)
        .collect();
    arguments.sort_by(|left, right| left.name.cmp(&right.name));
    arguments
}

fn collect_options(command: &Command) -> Vec<OptionInfo> {
    let mut options: Vec<_> = command
        .get_arguments()
        .filter(|arg| !arg.is_positional() && !arg.is_hide_set())
        .map(option_from_clap)
        .collect();
    options.sort_by(|left, right| left.flag.cmp(&right.flag));
    options
}

fn argument_from_clap(arg: &Arg) -> ArgumentInfo {
    let description = arg
        .get_long_help()
        .or_else(|| arg.get_help())
        .map(|help| help.to_string())
        .unwrap_or_default();

    let name = arg
        .get_value_names()
        .and_then(|names| names.first().map(ToString::to_string))
        .unwrap_or_else(|| arg.get_id().to_string());

    let info = ArgumentInfo::new(name, description);
    if arg.is_required_set() {
        info
    } else {
        info.optional()
    }
}

fn option_from_clap(arg: &Arg) -> OptionInfo {
    let takes_value = matches!(arg.get_action(), ArgAction::Set | ArgAction::Append);
    let description = arg
        .get_long_help()
        .or_else(|| arg.get_help())
        .map(|help| help.to_string())
        .unwrap_or_default();

    let long_flag = arg.get_long().map(|long| {
        if takes_value {
            if let Some(value_name) = option_value_name(arg) {
                return format!("--{long} <{value_name}>");
            }
        }
        format!("--{long}")
    });

    let short_flag = arg.get_short().map(|short| {
        if takes_value {
            if let Some(value_name) = option_value_name(arg) {
                return format!("-{short} <{value_name}>");
            }
        }
        format!("-{short}")
    });

    let mut option = OptionInfo::new(
        long_flag
            .clone()
            .or(short_flag.clone())
            .unwrap_or_else(|| arg.get_id().to_string()),
        description,
    );

    if let Some(short) = short_flag {
        if long_flag.is_some() {
            option = option.with_short(short);
        }
    }

    let default_values = arg
        .get_default_values()
        .iter()
        .map(|value| value.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    if takes_value && !default_values.is_empty() {
        option = option.with_default(default_values.join(", "));
    }

    let possible_values = arg
        .get_possible_values()
        .into_iter()
        .filter(|value| !value.is_hide_set())
        .map(|value| value.get_name().to_string())
        .collect::<Vec<_>>();
    if takes_value && !possible_values.is_empty() {
        option = option.with_possible(possible_values);
    }

    option
}

fn option_value_name(arg: &Arg) -> Option<String> {
    arg.get_value_names()
        .and_then(|names| names.first().map(ToString::to_string))
        .or_else(|| {
            if matches!(arg.get_action(), ArgAction::Set | ArgAction::Append) {
                Some(arg.get_id().to_string().to_ascii_uppercase())
            } else {
                None
            }
        })
}

fn command_summary(command: &Command) -> String {
    command
        .get_long_about()
        .or_else(|| command.get_about())
        .map(|about| about.to_string())
        .unwrap_or_default()
}

fn default_try_next(command: &Command, command_path: &str) -> Vec<RelatedCommand> {
    let mut commands = Vec::new();

    if let Some(first_subcommand) = command.get_subcommands().next() {
        commands.push(RelatedCommand::new(
            format!("{command_path} {} --help", first_subcommand.get_name()),
            "Inspect the first available subcommand.",
        ));
    }

    if !command
        .get_arguments()
        .any(|arg| arg.get_long() == Some("output") || arg.get_short() == Some('o'))
    {
        return commands;
    }

    commands.push(RelatedCommand::new(
        format!("{command_path} --output json"),
        "Render this surface as JSON.",
    ));

    commands
}

fn parent_command(command_path: &str) -> Option<String> {
    let mut segments = command_path
        .split_whitespace()
        .map(str::to_string)
        .collect::<Vec<_>>();
    if segments.len() <= 1 {
        return None;
    }
    segments.pop();
    Some(segments.join(" "))
}

fn format_option_flags(option: &OptionInfo) -> String {
    match &option.short {
        Some(short) => format!("{short}, {}", option.flag),
        None => option.flag.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Arg, ArgAction, Command};

    fn sample_command() -> Command {
        Command::new("test")
            .about("A test command")
            .arg(
                Arg::new("input")
                    .help("Input value")
                    .required(true)
                    .value_name("INPUT"),
            )
            .arg(
                Arg::new("output")
                    .long("output")
                    .short('o')
                    .help("Output format")
                    .default_value("yaml")
                    .value_parser(["yaml", "json", "table"]),
            )
            .arg(
                Arg::new("verbose")
                    .long("verbose")
                    .short('v')
                    .help("Enable verbose output")
                    .action(ArgAction::SetTrue),
            )
            .subcommand(Command::new("child").about("Child command"))
    }

    #[test]
    fn test_help_document_from_clap() {
        let help = HelpDocument::from_clap(&sample_command(), "test");

        assert_eq!(help.command, "test");
        assert_eq!(help.summary, "A test command");
        assert_eq!(help.arguments.len(), 1);
        assert!(
            help.options
                .iter()
                .any(|option| option.flag.contains("--output"))
        );
        assert!(
            help.before_you_run_it
                .iter()
                .any(|note| note.contains("Default structured format"))
        );
    }

    #[test]
    fn test_format_help_table() {
        let help = HelpDocument::from_clap(&sample_command(), "test");
        let formatted = format_help_table(&help);

        assert!(formatted.contains("NAME"));
        assert!(formatted.contains("test - A test command"));
        assert!(formatted.contains("SYNOPSIS"));
        assert!(formatted.contains("OPTIONS"));
    }
}
