//! Notification commands

use crate::output::{CommandOutput, NextStep, OutputEnvelope};
use jellyfin_api::JellyfinClient;
use jellyfin_core::Result;

/// List notifications
pub async fn list(profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;
    let result = client.get_notifications().await?;
    let value = serde_json::to_value(result.items)?;
    let count = result.total_record_count.unwrap_or(0) as usize;

    let envelope: CommandOutput = OutputEnvelope::success(
        "jellyfin notifications list",
        format!("{} notifications", count),
    )
    .with_data(value)
    .with_next_step(NextStep::new(
        "mark_read",
        "jellyfin notifications mark-read <NOTIFICATION_ID>",
        "Mark notification as read",
    ));

    Ok(envelope)
}

/// Mark notification as read
pub async fn mark_read(notification_id: String, profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;
    client.mark_notification_read(&notification_id).await?;

    let envelope: CommandOutput = OutputEnvelope::success(
        "jellyfin notifications mark-read",
        format!("Marked notification {} as read", notification_id),
    )
    .with_next_step(NextStep::new(
        "list_notifications",
        "jellyfin notifications list",
        "List all notifications",
    ));

    Ok(envelope)
}

/// Mark all notifications as read
pub async fn mark_all_read(profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;
    client.mark_all_notifications_read().await?;

    let envelope: CommandOutput = OutputEnvelope::success(
        "jellyfin notifications mark-all-read",
        "Marked all notifications as read",
    )
    .with_next_step(NextStep::new(
        "list_notifications",
        "jellyfin notifications list",
        "List all notifications",
    ));

    Ok(envelope)
}

/// Handle notification subcommands
pub async fn handle(
    action: crate::NotificationCommands,
    profile: Option<&str>,
) -> Result<CommandOutput> {
    match action {
        crate::NotificationCommands::List => list(profile).await,
        crate::NotificationCommands::MarkRead { notification_id } => {
            mark_read(notification_id, profile).await
        }
        crate::NotificationCommands::MarkAllRead => mark_all_read(profile).await,
    }
}
