//! E2E testing commands
//!
//! This module provides commands for setting up and managing
//! end-to-end testing with a local Jellyfin server.

pub mod config;
pub mod logs;
pub mod media;
pub mod reset;
pub mod server;
pub mod setup;
pub mod status;

use clap::Subcommand;
use jellyfin_core::Result;

use crate::output::CommandOutput;

/// E2E testing commands
#[derive(Subcommand, Debug, Clone)]
pub enum E2ECommands {
    /// Initialize E2E test environment with isolated Jellyfin configuration
    Setup(setup::SetupOptions),

    /// Start the E2E Jellyfin server
    Start(server::StartOptions),

    /// Stop the E2E Jellyfin server
    Stop(server::StopOptions),

    /// Manage test media files
    #[command(subcommand)]
    Media(media::MediaCommands),

    /// Show E2E environment status
    Status(status::StatusOptions),

    /// Show server logs
    #[command(subcommand)]
    Logs(logs::LogsCommands),

    /// Reset E2E environment
    #[command(subcommand)]
    Reset(reset::ResetCommands),

    /// Configure E2E environment
    #[command(subcommand)]
    Config(config::ConfigCommands),
}

impl E2ECommands {
    /// Execute the E2E command
    pub fn execute(&self) -> Result<CommandOutput> {
        match self {
            E2ECommands::Setup(opts) => opts.execute(),
            E2ECommands::Start(opts) => opts.execute(),
            E2ECommands::Stop(opts) => opts.execute(),
            E2ECommands::Media(cmd) => cmd.execute(),
            E2ECommands::Status(opts) => opts.execute(),
            E2ECommands::Logs(cmd) => cmd.execute(),
            E2ECommands::Reset(cmd) => cmd.execute(),
            E2ECommands::Config(cmd) => cmd.execute(),
        }
    }
}
