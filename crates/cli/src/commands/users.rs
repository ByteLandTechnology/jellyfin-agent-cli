//! User commands

use crate::output::{CommandOutput, NextStep, OutputEnvelope};
use jellyfin_api::JellyfinClient;
use jellyfin_core::Result;

/// Handle user subcommands
pub async fn handle(action: crate::UserCommands, profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;

    match action {
        crate::UserCommands::List => {
            let users = client.get_users().await?;
            let users_value = serde_json::to_value(users)?;
            let count = users_value.as_array().map(|a| a.len()).unwrap_or(0);

            let envelope: CommandOutput =
                OutputEnvelope::success("jellyfin users list", format!("{} users", count))
                    .with_data(users_value)
                    .with_next_step(NextStep::new(
                        "get_user",
                        "jellyfin users get <USER_ID>",
                        "Get user details",
                    ))
                    .with_next_step(NextStep::new(
                        "create_user",
                        "jellyfin users create <NAME>",
                        "Create a new user",
                    ));

            Ok(envelope)
        }
        crate::UserCommands::Get { user_id } => {
            let user = if let Some(id) = user_id {
                client.get_user(&id).await?
            } else {
                client.get_current_user().await?
            };
            let user_value = serde_json::to_value(user)?;

            let envelope: CommandOutput =
                OutputEnvelope::success("jellyfin users get", "User information retrieved")
                    .with_data(user_value)
                    .with_next_step(NextStep::new(
                        "update_user",
                        "jellyfin users update <USER_ID>",
                        "Update user settings",
                    ))
                    .with_next_step(NextStep::new(
                        "view_activity",
                        "jellyfin users activity <USER_ID>",
                        "View user activity log",
                    ));

            Ok(envelope)
        }
        crate::UserCommands::Create { name, password } => {
            let user = client.create_user(&name, &password).await?;
            let user_value = serde_json::to_value(&user)?;

            let envelope: CommandOutput = OutputEnvelope::success(
                "jellyfin users create",
                format!("Created user '{}'", name),
            )
            .with_data(user_value)
            .with_next_step(NextStep::new(
                "set_password",
                format!("jellyfin users password {} <NEW_PASSWORD>", user.id),
                "Set user password",
            ))
            .with_next_step(NextStep::new(
                "set_policy",
                format!("jellyfin users policy {}", user.id),
                "Configure user access policy",
            ));

            Ok(envelope)
        }
        crate::UserCommands::Delete { user_id } => {
            client.delete_user(&user_id).await?;

            let envelope: CommandOutput = OutputEnvelope::success(
                "jellyfin users delete",
                format!("Deleted user '{}'", user_id),
            )
            .with_next_step(NextStep::new(
                "list_users",
                "jellyfin users list",
                "List remaining users",
            ));

            Ok(envelope)
        }
    }
}
