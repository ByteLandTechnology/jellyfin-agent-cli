//! Full-featured command line client for Jellyfin.

use clap::{
    ArgAction, ArgMatches, Command, CommandFactory, FromArgMatches, Parser, Subcommand,
    error::ErrorKind,
};
use jellyfin_core::{Config, JellyfinError};
use std::{ffi::OsString, process::ExitCode};

mod commands;
mod output;
mod repl;

use output::{ActiveContextState, CommandOutput, HelpDocument, OutputFormat, StructuredHelpFormat};

/// Full-featured command line client for Jellyfin media browsing, playback,
/// administration, and automation workflows.
#[derive(Parser, Debug)]
#[command(name = "jellyfin-agent-cli")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
#[command(disable_help_subcommand = true)]
#[command(
    about = "Full-featured command line client for Jellyfin media browsing, playback, administration, and automation workflows."
)]
struct Cli {
    /// Output format (default: yaml; supported: table, yaml, toml, json, ndjson)
    #[arg(
        short = 'o',
        long,
        global = true,
        value_name = "FORMAT",
        value_parser = ["table", "yaml", "toml", "json", "ndjson"]
    )]
    output: Option<String>,

    /// Server URL (overrides config)
    #[arg(short = 's', long, global = true)]
    server: Option<String>,

    /// Server profile name from config
    #[arg(short = 'P', long, global = true)]
    profile: Option<String>,

    /// Enable debug output
    #[arg(short = 'd', long, global = true)]
    debug: bool,

    #[command(subcommand)]
    command: Commands,
}

/// Available commands
#[derive(Subcommand, Debug)]
enum Commands {
    /// Login to Jellyfin server
    Login {
        /// Server URL
        #[arg(short, long)]
        server: Option<String>,
        /// Username
        #[arg(short, long)]
        username: Option<String>,
        /// Password
        #[arg(short, long)]
        password: Option<String>,
        /// Save with this profile name
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Logout and clear credentials
    Logout,

    /// Search for media
    Search {
        /// Search query
        query: String,

        /// Limit results
        #[arg(short, long)]
        limit: Option<u32>,
    },

    /// Play a media item
    Play {
        /// Item ID to play
        item_id: String,

        /// External player command
        #[arg(short = 'e', long)]
        player: Option<String>,

        /// Print URL instead of playing
        #[arg(long)]
        print_url: bool,

        /// Start from position (HH:MM:SS or seconds)
        #[arg(short = 'p', long)]
        position: Option<String>,
    },

    /// Pause playback
    Pause,

    /// Resume playback
    Resume,

    /// Continue watching
    Continue {
        /// Limit results
        #[arg(short, long)]
        limit: Option<u32>,
    },

    /// Latest media
    Latest {
        /// Limit results
        #[arg(short, long)]
        limit: Option<u32>,
    },

    /// List libraries
    Libraries {
        #[command(subcommand)]
        action: LibraryCommands,
    },

    /// Item operations
    Items {
        #[command(subcommand)]
        action: ItemCommands,
    },

    /// User operations
    Users {
        #[command(subcommand)]
        action: UserCommands,
    },

    /// Playback operations
    Playback {
        #[command(subcommand)]
        action: PlaybackCommands,
    },

    /// Server info
    Info,

    /// Server statistics
    Stats,

    /// Configuration management
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },

    /// Active context (server profile) management
    Context {
        #[command(subcommand)]
        action: ContextCommands,
    },

    /// System operations (restart, shutdown)
    System {
        #[command(subcommand)]
        action: SystemCommands,
    },

    /// Scheduled tasks management
    ScheduledTasks {
        #[command(subcommand)]
        action: ScheduledTaskCommands,
    },

    /// Device management
    Devices {
        #[command(subcommand)]
        action: DeviceCommands,
    },

    /// Playlist management
    Playlists {
        #[command(subcommand)]
        action: PlaylistCommands,
    },

    /// Notification management
    Notifications {
        #[command(subcommand)]
        action: NotificationCommands,
    },

    /// Plugin management
    Plugins {
        #[command(subcommand)]
        action: PluginCommands,
    },

    /// Channel management
    Channels {
        #[command(subcommand)]
        action: ChannelCommands,
    },

    /// Session management
    Sessions,

    /// Activity log
    ActivityLog,

    /// Remote search
    RemoteSearch {
        /// Search query
        query: String,

        /// Item type (Movie, Series, Music, etc.)
        #[arg(long)]
        item_type: Option<String>,

        /// Year
        #[arg(long)]
        year: Option<u32>,
    },

    /// List all genres
    Genres,

    /// List all studios
    Studios,

    /// List all actors/artists
    Actors,

    /// Render structured help for the CLI or a specific command path
    Help {
        /// Command path to inspect (for example: e2e logs)
        #[arg(value_name = "COMMAND_PATH", num_args = 0..)]
        command_path: Vec<String>,

        /// Structured help format
        #[arg(long, value_name = "FORMAT", default_value = "yaml", value_parser = ["yaml", "json", "toml"])]
        format: String,
    },

    /// End-to-end testing commands
    E2E {
        #[command(subcommand)]
        action: commands::e2e::E2ECommands,
    },

    /// Start interactive REPL mode
    Repl,
}

/// Config subcommands
#[derive(Subcommand, Debug)]
enum ConfigCommands {
    /// Show current configuration
    Show,

    /// List configured servers
    ListServers,

    /// Set default server
    SetDefault {
        /// Profile name to mark as default
        #[arg(long, value_name = "NAME")]
        name: String,
    },

    /// Remove a server configuration
    RemoveServer {
        /// Profile name to remove
        #[arg(long, value_name = "NAME")]
        name: String,
    },
}

/// Active context subcommands
#[derive(Subcommand, Debug)]
enum ContextCommands {
    /// Show current active context (server profile)
    Show,

    /// Switch to a different context
    Use {
        /// Context (server profile) name to switch to
        name: String,
    },
}

/// System subcommands
#[derive(Subcommand, Debug)]
enum SystemCommands {
    /// Restart the server
    Restart,

    /// Shutdown the server
    Shutdown,
}

/// Scheduled tasks subcommands
#[derive(Subcommand, Debug)]
enum ScheduledTaskCommands {
    /// List all scheduled tasks
    List,

    /// Get task details
    Get {
        /// Task ID
        task_id: String,
    },

    /// Start a task
    Start {
        /// Task ID
        task_id: String,
    },

    /// Stop a running task
    Stop {
        /// Task ID
        task_id: String,
    },
}

/// Device subcommands
#[derive(Subcommand, Debug)]
enum DeviceCommands {
    /// List all devices
    List,

    /// Get device details
    Get {
        /// Device ID
        device_id: String,
    },
}

/// Playlist subcommands
#[derive(Subcommand, Debug)]
enum PlaylistCommands {
    /// List all playlists
    List,

    /// Get playlist details
    Get {
        /// Playlist ID
        playlist_id: String,
    },

    /// Create a new playlist
    Create {
        /// Playlist name
        #[arg(long)]
        name: String,
    },

    /// Add items to playlist
    Add {
        /// Playlist ID
        #[arg(long)]
        playlist_id: String,

        /// Item IDs to add
        #[arg(long, num_args = 1..)]
        items: Vec<String>,
    },

    /// Remove items from playlist
    Remove {
        /// Playlist ID
        #[arg(long)]
        playlist_id: String,

        /// Item IDs to remove
        #[arg(long, num_args = 1..)]
        items: Vec<String>,
    },

    /// Delete a playlist
    Delete {
        /// Playlist ID
        playlist_id: String,
    },
}

/// Notification subcommands
#[derive(Subcommand, Debug)]
enum NotificationCommands {
    /// List notifications
    List,

    /// Mark notification as read
    MarkRead {
        /// Notification ID
        notification_id: String,
    },

    /// Mark all notifications as read
    MarkAllRead,
}

/// Plugin subcommands
#[derive(Subcommand, Debug)]
enum PluginCommands {
    /// List all plugins
    List,

    /// Get plugin details
    Get {
        /// Plugin ID
        plugin_id: String,
    },

    /// Uninstall a plugin
    Uninstall {
        /// Plugin ID
        plugin_id: String,
    },
}

/// Channel subcommands
#[derive(Subcommand, Debug)]
enum ChannelCommands {
    /// List all channels
    List,

    /// Get channel items
    Items {
        /// Channel ID
        channel_id: String,
    },
}

/// Library subcommands
#[derive(Subcommand, Debug)]
enum LibraryCommands {
    /// List all libraries
    List,

    /// Show items in a library
    Items {
        /// Library ID or name
        library: String,

        /// Limit results
        #[arg(short, long)]
        limit: Option<u32>,
    },

    /// Add a new media library
    Add {
        /// Library name
        #[arg(long)]
        name: String,

        /// Collection type (movies, tvshows, music, photos, books, homevideos, mixed)
        #[arg(long)]
        collection_type: String,

        /// Media paths (can be specified multiple times) - optional
        #[arg(long)]
        paths: Option<Vec<String>>,
    },

    /// Remove a media library
    Remove {
        /// Library name
        #[arg(long)]
        name: String,
    },
}

/// Item subcommands
#[derive(Subcommand, Debug)]
enum ItemCommands {
    /// List items
    List {
        /// Parent ID to list under
        #[arg(short, long)]
        parent: Option<String>,

        /// Recursively list
        #[arg(short, long)]
        recursive: bool,

        /// Sort by field
        #[arg(long)]
        sort_by: Option<String>,

        /// Limit results
        #[arg(short = 'n', long)]
        limit: Option<u32>,
    },

    /// Get item details
    Get {
        /// Item ID
        item_id: String,
    },

    /// Refresh item metadata
    Refresh {
        /// Item ID
        item_id: String,
    },

    /// Delete item
    Delete {
        /// Item ID
        item_id: String,
    },

    /// Add item to favorites
    Favorite {
        /// Item ID
        item_id: String,
    },

    /// Remove item from favorites
    Unfavorite {
        /// Item ID
        item_id: String,
    },

    /// List favorite items
    Favorites,

    /// Rate an item
    Rate {
        /// Item ID
        item_id: String,

        /// Rating value (1-10, or use 0 to unlike)
        #[arg(long)]
        rating: Option<f64>,
    },
}

/// User subcommands
#[derive(Subcommand, Debug)]
enum UserCommands {
    /// List users
    List,

    /// Get user info
    Get {
        /// User ID (omit for current user)
        user_id: Option<String>,
    },

    /// Create new user
    Create {
        /// Username
        name: String,

        /// Password
        password: String,
    },

    /// Delete user
    Delete {
        /// User ID
        user_id: String,
    },
}

/// Playback subcommands
#[derive(Subcommand, Debug)]
enum PlaybackCommands {
    /// Show current playback info
    Info,

    /// Seek to position
    Seek {
        /// Position in HH:MM:SS or seconds
        position: String,
    },

    /// Stop playback
    Stop,

    /// Show queue
    Queue,
}

pub async fn main_entry(raw_args: Vec<OsString>) -> ExitCode {
    let requested_output =
        detect_requested_output_format(&raw_args).unwrap_or_else(OutputFormat::default_structured);

    let mut command = Cli::command();
    let matches = match command.try_get_matches_from_mut(raw_args.clone()) {
        Ok(matches) => matches,
        Err(error) => return handle_parse_error(&command, &raw_args, requested_output, &error),
    };

    let cli = match Cli::from_arg_matches(&matches) {
        Ok(cli) => cli,
        Err(error) => return handle_parse_error(&command, &raw_args, requested_output, &error),
    };

    // REPL mode: intercept before normal command dispatch.
    if let Commands::Repl = &cli.command {
        let code = repl::run(cli.profile.clone(), cli.output.clone(), cli.debug).await;
        return ExitCode::from(code as u8);
    }

    if let Commands::Help {
        command_path,
        format,
    } = &cli.command
    {
        return render_structured_help_command(&command, command_path, format);
    }

    init_tracing(cli.debug);

    let command_path = matched_command_path(command.get_name(), &matches);
    let context_state = ActiveContextState::from_runtime(&cli.command, cli.profile.as_deref());
    let output_format = match resolve_output_format(&cli) {
        Ok(format) => format,
        Err(error) => {
            let envelope = finalize_envelope(
                CommandOutput::from_command_error(command_path, &error),
                context_state,
            );
            let text = output::format_output(&envelope, OutputFormat::default_structured());
            write_rendered(&text, false);
            return ExitCode::from(error.exit_code() as u8);
        }
    };

    let (envelope, exit_code) = match execute_command(cli).await {
        Ok(envelope) => {
            let exit_code = match envelope.status {
                output::Status::Success => 0,
                output::Status::Error => 1,
            };
            (envelope, exit_code)
        }
        Err(error) => (
            CommandOutput::from_command_error(command_path, &error),
            error.exit_code(),
        ),
    };

    let envelope = finalize_envelope(envelope, context_state);
    let text = output::format_output(&envelope, output_format);
    write_rendered(&text, false);
    ExitCode::from(exit_code as u8)
}

pub(crate) fn init_tracing(debug: bool) {
    if debug {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::WARN)
            .init();
    }
}

pub(crate) async fn execute_command(cli: Cli) -> jellyfin_core::Result<CommandOutput> {
    let Cli {
        output: _,
        server: _,
        profile,
        debug: _,
        command,
    } = cli;

    match command {
        Commands::Login {
            server,
            username,
            password,
            name,
        } => commands::auth::login(server, username, password, name).await,
        Commands::Logout => commands::auth::logout().await,
        Commands::Search { query, limit } => {
            commands::search::search(query, limit, profile.as_deref()).await
        }
        Commands::Play {
            item_id,
            player,
            print_url,
            position,
        } => {
            commands::playback::play(item_id, player, print_url, position, profile.as_deref()).await
        }
        Commands::Pause => commands::playback::pause(profile.as_deref()).await,
        Commands::Resume => commands::playback::resume(profile.as_deref()).await,
        Commands::Continue { limit } => {
            commands::playback::continue_watching(limit, profile.as_deref()).await
        }
        Commands::Latest { limit } => commands::items::latest(limit, profile.as_deref()).await,
        Commands::Libraries { action } => {
            commands::library::handle(action, profile.as_deref()).await
        }
        Commands::Items { action } => commands::items::handle(action, profile.as_deref()).await,
        Commands::Users { action } => commands::users::handle(action, profile.as_deref()).await,
        Commands::Playback { action } => {
            commands::playback::handle(action, profile.as_deref()).await
        }
        Commands::Info => commands::system::info(profile.as_deref()).await,
        Commands::Stats => commands::stats::info(profile.as_deref()).await,
        Commands::Config { action } => match action {
            ConfigCommands::Show => commands::config::show().await,
            ConfigCommands::ListServers => commands::config::list_servers().await,
            ConfigCommands::SetDefault { name } => commands::config::set_default(name).await,
            ConfigCommands::RemoveServer { name } => commands::config::remove_server(name).await,
        },
        Commands::Context { action } => match action {
            ContextCommands::Show => commands::context::show().await,
            ContextCommands::Use { name } => commands::context::use_context(name).await,
        },
        Commands::System { action } => commands::system::handle(action, profile.as_deref()).await,
        Commands::ScheduledTasks { action } => {
            commands::scheduled_tasks::handle(action, profile.as_deref()).await
        }
        Commands::Devices { action } => commands::devices::handle(action, profile.as_deref()).await,
        Commands::Playlists { action } => {
            commands::playlists::handle(action, profile.as_deref()).await
        }
        Commands::Notifications { action } => {
            commands::notifications::handle(action, profile.as_deref()).await
        }
        Commands::Plugins { action } => commands::plugins::handle(action, profile.as_deref()).await,
        Commands::Channels { action } => {
            commands::channels::handle(action, profile.as_deref()).await
        }
        Commands::Sessions => commands::sessions::list(profile.as_deref()).await,
        Commands::ActivityLog => commands::activity_log::entries(profile.as_deref()).await,
        Commands::RemoteSearch {
            query,
            item_type,
            year,
        } => commands::remote_search::search(query, item_type, year, profile.as_deref()).await,
        Commands::Genres => commands::media::genres(profile.as_deref()).await,
        Commands::Studios => commands::media::studios(profile.as_deref()).await,
        Commands::Actors => commands::media::actors(profile.as_deref()).await,
        Commands::Help { .. } => unreachable!("help is handled before execute_command"),
        Commands::E2E { action } => action.execute(),
        Commands::Repl => unreachable!("repl is handled before execute_command"),
    }
}

pub(crate) fn resolve_output_format(cli: &Cli) -> jellyfin_core::Result<OutputFormat> {
    if let Some(format) = cli.output.as_deref() {
        return OutputFormat::from_str(format).ok_or_else(|| {
            jellyfin_core::JellyfinError::invalid_input(
                "output format",
                format!(
                    "must be one of: {}",
                    OutputFormat::supported_values().join(", ")
                ),
            )
        });
    }

    if let Ok(config) = Config::load() {
        return OutputFormat::from_str(&config.output.format).ok_or_else(|| {
            jellyfin_core::JellyfinError::invalid_input(
                "output format",
                format!(
                    "unsupported config value '{}'; expected one of: {}",
                    config.output.format,
                    OutputFormat::supported_values().join(", ")
                ),
            )
        });
    }

    Ok(OutputFormat::default_structured())
}

pub(crate) fn handle_parse_error(
    command: &Command,
    raw_args: &[OsString],
    output_format: OutputFormat,
    error: &clap::Error,
) -> ExitCode {
    if error.kind() == ErrorKind::DisplayVersion {
        write_rendered(&error.render().to_string(), error.use_stderr());
        return ExitCode::from(error.exit_code() as u8);
    }

    let command_path = command_path_from_args(command, raw_args);
    let target_command = find_command_by_path(command, &command_path);

    if should_render_human_help(error, target_command) {
        let help = build_help_document(command, raw_args, None);
        let text = output::format_help_human(&help);
        write_rendered(&text, false);
        return ExitCode::from(0);
    }

    let parse_error = parse_error_to_command_error(error);
    let envelope = CommandOutput::from_command_error(command_path, &parse_error);
    let text = output::format_output(&envelope, output_format);
    write_rendered(&text, false);
    ExitCode::from(parse_error.exit_code() as u8)
}

fn build_help_document(
    root_command: &Command,
    raw_args: &[OsString],
    error: Option<&clap::Error>,
) -> HelpDocument {
    let command_path = command_path_from_args(root_command, raw_args);
    let command = find_command_by_path(root_command, &command_path);
    let mut help = HelpDocument::from_clap(command, command_path.clone());

    if let Some(error) = error {
        if !matches!(
            error.kind(),
            ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
        ) {
            if let Some(summary) = parse_error_summary(error) {
                help = help.push_before_you_run_it(format!("Usage check failed: {summary}"));
            }
        }
    }

    help
}

pub(crate) fn render_structured_help_command(
    root_command: &Command,
    command_path: &[String],
    format: &str,
) -> ExitCode {
    let output_format = match StructuredHelpFormat::from_str(format) {
        Some(format) => format.into_output_format(),
        None => {
            let error =
                JellyfinError::invalid_input("help format", "must be one of: yaml, json, toml");
            let envelope = CommandOutput::from_command_error("jellyfin-agent-cli help", &error);
            let text = output::format_output(&envelope, OutputFormat::default_structured());
            write_rendered(&text, false);
            return ExitCode::from(error.exit_code() as u8);
        }
    };

    let resolved_path = resolve_help_command_path(root_command.get_name(), command_path);
    let Some(command) = find_command_by_segments(root_command, command_path) else {
        let error = JellyfinError::invalid_input(
            "command path",
            format!(
                "unknown command path '{}'",
                if command_path.is_empty() {
                    root_command.get_name().to_string()
                } else {
                    command_path.join(" ")
                }
            ),
        );
        let envelope = CommandOutput::from_command_error("jellyfin-agent-cli help", &error);
        let text = output::format_output(&envelope, output_format);
        write_rendered(&text, false);
        return ExitCode::from(error.exit_code() as u8);
    };

    let help = HelpDocument::from_clap(command, resolved_path);
    let text = output::format_help_document(&help, output_format);
    write_rendered(&text, false);
    ExitCode::from(0)
}

fn resolve_help_command_path(root_name: &str, command_path: &[String]) -> String {
    if command_path.is_empty() {
        root_name.to_string()
    } else {
        format!("{root_name} {}", command_path.join(" "))
    }
}

fn finalize_envelope(envelope: CommandOutput, context_state: ActiveContextState) -> CommandOutput {
    envelope
        .normalize_public_commands("jellyfin-agent-cli")
        .with_active_context(context_state)
}

fn should_render_human_help(error: &clap::Error, target_command: &Command) -> bool {
    if error.kind() == ErrorKind::DisplayHelp {
        return true;
    }

    if target_command.get_subcommands().next().is_none() {
        return false;
    }

    matches!(
        error.kind(),
        ErrorKind::MissingSubcommand | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
    )
}

fn parse_error_to_command_error(error: &clap::Error) -> JellyfinError {
    let summary =
        parse_error_summary(error).unwrap_or_else(|| "Invalid command usage.".to_string());

    JellyfinError::invalid_input("arguments", summary).with_hint(
        "Use '--help' for human-readable help or 'help --format yaml' for structured help.",
    )
}

impl ActiveContextState {
    fn from_runtime(command: &Commands, requested_profile: Option<&str>) -> Self {
        let config = Config::load().ok();
        let persisted_profile = config
            .as_ref()
            .and_then(|cfg| (!cfg.default_server.is_empty()).then(|| cfg.default_server.clone()));
        let requested_profile = requested_profile.map(str::to_string);
        let requested_profile_found = requested_profile.as_ref().map(|profile| {
            config
                .as_ref()
                .is_some_and(|cfg| cfg.servers.contains_key(profile))
        });
        let uses_override = command_uses_profile_override(command);
        let override_applied = uses_override && requested_profile_found == Some(true);
        let effective_profile = if override_applied {
            requested_profile.clone()
        } else {
            persisted_profile.clone()
        };

        let note = match (
            requested_profile.as_deref(),
            requested_profile_found,
            uses_override,
        ) {
            (Some(profile), Some(false), false) => Some(format!(
                "Requested profile override '{profile}' is not configured; this command reports the persisted active context."
            )),
            (Some(profile), Some(true), false) => Some(format!(
                "Requested profile override '{profile}' does not mutate persisted context; this command reports the persisted active context."
            )),
            (Some(profile), Some(false), true) => Some(format!(
                "Requested profile override '{profile}' is not configured."
            )),
            _ => None,
        };

        let mut state = ActiveContextState::new(
            "explicit --profile override wins for that invocation and never mutates persisted context",
        );
        state.persisted_profile = persisted_profile;
        state.requested_profile = requested_profile;
        state.requested_profile_found = requested_profile_found;
        state.effective_profile = effective_profile;
        state.override_applied = override_applied;
        state.note = note;
        state
    }
}

fn parse_error_summary(error: &clap::Error) -> Option<String> {
    let mut lines = Vec::new();

    for line in error.to_string().lines() {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with("Usage:")
            || trimmed.starts_with("For more information")
        {
            if trimmed.starts_with("Usage:") || trimmed.starts_with("For more information") {
                break;
            }
            continue;
        }

        lines.push(trimmed.trim_start_matches("error: ").to_string());
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines.join(" "))
    }
}

pub(crate) fn detect_requested_output_format(args: &[OsString]) -> Option<OutputFormat> {
    let mut expect_value = false;

    for argument in args.iter().skip(1) {
        let Some(token) = argument.to_str() else {
            continue;
        };

        if expect_value {
            return OutputFormat::from_str(token);
        }

        if token == "--output" || token == "-o" {
            expect_value = true;
            continue;
        }

        if let Some(value) = token.strip_prefix("--output=") {
            return OutputFormat::from_str(value);
        }

        if let Some(value) = token.strip_prefix("-o") {
            if !value.is_empty() {
                return OutputFormat::from_str(value);
            }
        }
    }

    None
}

fn matched_command_path(root_name: &str, matches: &ArgMatches) -> String {
    let mut parts = vec![root_name.to_string()];
    let mut current = matches;

    while let Some((name, submatches)) = current.subcommand() {
        if name != "help" {
            parts.push(name.to_string());
        }
        current = submatches;
    }

    parts.join(" ")
}

fn command_path_from_args(root_command: &Command, raw_args: &[OsString]) -> String {
    let mut current = root_command;
    let mut parts = vec![root_command.get_name().to_string()];
    let mut skip_next = false;

    for argument in raw_args.iter().skip(1) {
        if skip_next {
            skip_next = false;
            continue;
        }

        let Some(token) = argument.to_str() else {
            continue;
        };

        if token == "--" {
            break;
        }

        if let Some(should_skip_next) = option_token_consumption(root_command, current, token) {
            skip_next = should_skip_next;
            continue;
        }

        if token == "help" {
            continue;
        }

        if let Some(subcommand) = current.find_subcommand(token) {
            if subcommand.get_name() != "help" {
                current = subcommand;
                parts.push(subcommand.get_name().to_string());
            }
        }
    }

    parts.join(" ")
}

fn option_token_consumption(
    root_command: &Command,
    current: &Command,
    token: &str,
) -> Option<bool> {
    if let Some(rest) = token.strip_prefix("--") {
        if rest.is_empty() {
            return None;
        }

        let (name, has_inline_value) = match rest.split_once('=') {
            Some((name, _)) => (name, true),
            None => (rest, false),
        };

        if let Some(argument) = find_argument_by_long(root_command, current, name) {
            return Some(argument_takes_value(argument) && !has_inline_value);
        }
    }

    if token.starts_with('-') && !token.starts_with("--") && token.len() > 1 {
        let mut chars = token[1..].chars();
        if let Some(short) = chars.next() {
            if let Some(argument) = find_argument_by_short(root_command, current, short) {
                let has_inline_value = token.len() > 2;
                return Some(argument_takes_value(argument) && !has_inline_value);
            }
        }
    }

    None
}

fn find_argument_by_long<'a>(
    root_command: &'a Command,
    current: &'a Command,
    long: &str,
) -> Option<&'a clap::Arg> {
    current
        .get_arguments()
        .find(|argument| argument.get_long() == Some(long))
        .or_else(|| {
            root_command
                .get_arguments()
                .find(|argument| argument.get_long() == Some(long))
        })
}

fn find_argument_by_short<'a>(
    root_command: &'a Command,
    current: &'a Command,
    short: char,
) -> Option<&'a clap::Arg> {
    current
        .get_arguments()
        .find(|argument| argument.get_short() == Some(short))
        .or_else(|| {
            root_command
                .get_arguments()
                .find(|argument| argument.get_short() == Some(short))
        })
}

fn argument_takes_value(argument: &clap::Arg) -> bool {
    matches!(argument.get_action(), ArgAction::Set | ArgAction::Append)
}

fn find_command_by_path<'a>(root_command: &'a Command, command_path: &str) -> &'a Command {
    let mut current = root_command;

    for segment in command_path.split_whitespace().skip(1) {
        if let Some(subcommand) = current.find_subcommand(segment) {
            current = subcommand;
        } else {
            break;
        }
    }

    current
}

fn find_command_by_segments<'a>(
    root_command: &'a Command,
    segments: &[String],
) -> Option<&'a Command> {
    let mut current = root_command;

    for segment in segments {
        current = current.find_subcommand(segment)?;
    }

    Some(current)
}

fn command_uses_profile_override(command: &Commands) -> bool {
    matches!(
        command,
        Commands::Search { .. }
            | Commands::Play { .. }
            | Commands::Pause
            | Commands::Resume
            | Commands::Continue { .. }
            | Commands::Latest { .. }
            | Commands::Libraries { .. }
            | Commands::Items { .. }
            | Commands::Users { .. }
            | Commands::Playback { .. }
            | Commands::Info
            | Commands::Stats
    )
}

fn write_rendered(text: &str, stderr: bool) {
    if stderr {
        if text.ends_with('\n') {
            eprint!("{text}");
        } else {
            eprintln!("{text}");
        }
    } else if text.ends_with('\n') {
        print!("{text}");
    } else {
        println!("{text}");
    }
}
