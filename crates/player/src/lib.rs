//! External player integration for Jellyfin CLI.
//!
//! Handles launching external media players with proper URLs and options.

use jellyfin_core::{JellyfinError, Result};
use std::path::Path;
use std::process::Command;

/// Supported external players
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerType {
    /// mpv - highly recommended
    Mpv,
    /// VLC media player
    Vlc,
    /// ffplay - from FFmpeg
    Ffplay,
    /// Custom command
    Custom,
}

impl PlayerType {
    /// Parse from string
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "mpv" => Some(Self::Mpv),
            "vlc" => Some(Self::Vlc),
            "ffplay" => Some(Self::Ffplay),
            _ if !s.is_empty() => Some(Self::Custom),
            _ => None,
        }
    }

    /// Check if player is available
    pub fn is_available(&self) -> bool {
        let command = self.command();
        Path::new(&command).exists() || Self::which(&command)
    }

    /// Get the command name
    pub fn command(&self) -> String {
        match self {
            Self::Mpv => "mpv".to_string(),
            Self::Vlc => "vlc".to_string(),
            Self::Ffplay => "ffplay".to_string(),
            Self::Custom => "custom".to_string(),
        }
    }

    /// Check if command exists in PATH
    fn which(command: &str) -> bool {
        Command::new("which")
            .arg(command)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

/// Player configuration
#[derive(Debug, Clone)]
pub struct PlayerConfig {
    /// Player type
    pub player_type: PlayerType,
    /// Custom command (for Custom type)
    pub custom_command: Option<String>,
    /// Default arguments
    pub args: Vec<String>,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            player_type: PlayerType::Mpv,
            custom_command: None,
            args: vec!["--fs".to_string(), "--no-osc".to_string()],
        }
    }
}

impl PlayerConfig {
    /// Create new config
    pub fn new(player_type: PlayerType) -> Self {
        Self {
            player_type,
            ..Default::default()
        }
    }

    /// Set custom command
    pub fn with_custom_command(mut self, command: String) -> Self {
        self.custom_command = Some(command);
        self
    }

    /// Set arguments
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Check if player is available
    pub fn is_available(&self) -> bool {
        self.player_type.is_available()
    }
}

/// Player controller
pub struct Player {
    config: PlayerConfig,
}

impl Player {
    /// Create new player
    pub fn new(config: PlayerConfig) -> Self {
        Self { config }
    }

    /// Create with default config
    pub fn default_player() -> Self {
        Self::new(PlayerConfig::default())
    }

    /// Check if player is available
    pub fn is_available(&self) -> bool {
        self.config.is_available()
    }

    /// Get recommended player
    pub fn recommend() -> Option<PlayerType> {
        [PlayerType::Mpv, PlayerType::Vlc, PlayerType::Ffplay]
            .iter()
            .find(|p| p.is_available())
            .copied()
    }

    /// Play a URL
    pub fn play(&self, url: &str) -> Result<PlayerHandle> {
        if !self.is_available() {
            return Err(JellyfinError::internal(format!(
                "Player '{}' is not available",
                self.config.player_type.command()
            )));
        }

        let command = self
            .config
            .custom_command
            .clone()
            .unwrap_or_else(|| self.config.player_type.command());

        let mut cmd = Command::new(&command);

        // Add player-specific arguments
        match self.config.player_type {
            PlayerType::Mpv => {
                // mpv handles URLs directly
                for arg in &self.config.args {
                    cmd.arg(arg);
                }
                cmd.arg("--title").arg("Jellyfin");
                cmd.arg(url);
            }
            PlayerType::Vlc => {
                // VLC needs --fullscreen before the URL
                cmd.arg("--fullscreen");
                for arg in &self.config.args {
                    cmd.arg(arg);
                }
                cmd.arg(url);
            }
            PlayerType::Ffplay => {
                cmd.arg("-window_title").arg("Jellyfin");
                cmd.arg("-autoexit");
                for arg in &self.config.args {
                    cmd.arg(arg);
                }
                cmd.arg(url);
            }
            PlayerType::Custom => {
                for arg in &self.config.args {
                    cmd.arg(arg);
                }
                cmd.arg(url);
            }
        }

        let child = cmd
            .spawn()
            .map_err(|e| JellyfinError::internal(format!("Failed to launch player: {}", e)))?;

        Ok(PlayerHandle { child: Some(child) })
    }

    /// Play from a specific position
    pub fn play_from(&self, url: &str, seconds: u64) -> Result<PlayerHandle> {
        if !self.is_available() {
            return Err(JellyfinError::internal("Player not available"));
        }

        let command = self
            .config
            .custom_command
            .clone()
            .unwrap_or_else(|| self.config.player_type.command());

        let mut cmd = Command::new(&command);

        // Add start position for supported players
        match self.config.player_type {
            PlayerType::Mpv => {
                for arg in &self.config.args {
                    cmd.arg(arg);
                }
                cmd.arg("--start").arg(format!("+{}", seconds));
                cmd.arg(url);
            }
            PlayerType::Vlc => {
                cmd.arg("--fullscreen");
                for arg in &self.config.args {
                    cmd.arg(arg);
                }
                cmd.arg("--start-time").arg(seconds.to_string());
                cmd.arg(url);
            }
            PlayerType::Ffplay => {
                cmd.arg("-ss").arg(seconds.to_string());
                cmd.arg("-window_title").arg("Jellyfin");
                cmd.arg("-autoexit");
                for arg in &self.config.args {
                    cmd.arg(arg);
                }
                cmd.arg(url);
            }
            PlayerType::Custom => {
                for arg in &self.config.args {
                    cmd.arg(arg);
                }
                cmd.arg(url);
            }
        }

        let child = cmd
            .spawn()
            .map_err(|e| JellyfinError::internal(format!("Failed to launch player: {}", e)))?;

        Ok(PlayerHandle { child: Some(child) })
    }
}

/// Handle to a running player process
pub struct PlayerHandle {
    child: Option<std::process::Child>,
}

impl PlayerHandle {
    /// Wait for player to exit
    pub fn wait(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            child
                .wait()
                .map_err(|e| JellyfinError::internal(format!("Player wait error: {}", e)))?;
        }
        Ok(())
    }

    /// Get player PID
    pub fn pid(&self) -> Option<u32> {
        self.child.as_ref().map(|c| c.id())
    }

    /// Try to kill the player
    pub fn kill(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            child
                .kill()
                .map_err(|e| JellyfinError::internal(format!("Failed to kill player: {}", e)))?;
        }
        Ok(())
    }
}

impl Drop for PlayerHandle {
    fn drop(&mut self) {
        // Don't kill on drop - let the user close the player naturally
        // The child process will be reaped by the OS
    }
}
