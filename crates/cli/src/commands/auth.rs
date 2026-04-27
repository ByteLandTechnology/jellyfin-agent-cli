//! Authentication commands

use crate::output::{CommandOutput, NextStep, OutputEnvelope};
use jellyfin_api::JellyfinClient;
use jellyfin_core::Result;
/// Login to Jellyfin server
pub async fn login(
    server: Option<String>,
    username: Option<String>,
    password: Option<String>,
    name: Option<String>,
) -> Result<CommandOutput> {
    use jellyfin_core::{Config, ServerConfig};
    use rpassword::prompt_password;

    // Get server URL
    let server_url = match server {
        Some(s) => s,
        None => {
            let config = Config::load()?;
            let server_name = name.as_ref().unwrap_or(&config.default_server);
            let server_config = config.get_server(server_name).ok_or_else(|| {
                jellyfin_core::JellyfinError::required_field(format!("server '{}'", server_name))
            })?;
            server_config.url.clone()
        }
    };

    // Get username
    let username = match username {
        Some(u) => u,
        None => {
            eprint!("Username: ");
            std::io::Write::flush(&mut std::io::stderr()).unwrap();
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            input.trim().to_string()
        }
    };

    // Get password
    let password = match password {
        Some(p) => p,
        None => prompt_password("Password: ").map_err(|e| {
            jellyfin_core::JellyfinError::internal(format!("Failed to read password: {}", e))
        })?,
    };

    // Authenticate
    let mut client = JellyfinClient::new(server_url)?;
    let auth = client.authenticate(&username, &password).await?;

    // Save credentials
    let profile_name = name.unwrap_or_else(|| {
        // Generate a name from the server URL
        "home".to_string()
    });

    let mut config = Config::load()?;
    let mut credentials = Config::load_credentials().unwrap_or_default();

    if let Some(token) = auth.access_token {
        // Update server config
        config.add_server(
            profile_name.clone(),
            ServerConfig::new(client.server_url().to_string(), username.clone()),
        );

        // Save token
        credentials.tokens.insert(profile_name.clone(), token);
        config.set_default_server(profile_name.clone());

        config.save()?;
        Config::save_credentials(&credentials)?;

        let envelope: CommandOutput =
            OutputEnvelope::success("jellyfin login", format!("Logged in as '{}'", username))
                .with_data(serde_json::json!({
                    "server": client.server_url(),
                    "username": username,
                    "profile": profile_name
                }))
                .with_next_step(NextStep::new(
                    "search_media",
                    "jellyfin search <query>",
                    "Search for media",
                ))
                .with_next_step(NextStep::new(
                    "view_libraries",
                    "jellyfin libraries list",
                    "Browse media libraries",
                ));

        Ok(envelope)
    } else {
        Err(jellyfin_core::JellyfinError::auth_failed(
            "No token received",
        ))
    }
}

/// Logout and clear credentials
pub async fn logout() -> Result<CommandOutput> {
    use jellyfin_core::Config;

    let mut config = Config::load()?;

    let envelope: CommandOutput =
        OutputEnvelope::success("jellyfin logout", "Logged out successfully")
            .with_next_step(NextStep::new(
                "login",
                "jellyfin login",
                "Login to a server",
            ))
            .with_next_step(NextStep::new(
                "view_config",
                "jellyfin config show",
                "View current configuration",
            ));

    // Clear default server but keep other config
    config.default_server = String::new();
    config.save()?;

    Ok(envelope)
}
