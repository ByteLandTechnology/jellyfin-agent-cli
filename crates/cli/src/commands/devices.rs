//! Device commands

use crate::output::{CommandOutput, NextStep, OutputEnvelope};
use jellyfin_api::JellyfinClient;
use jellyfin_core::Result;

/// List all devices
pub async fn list(profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;
    let result = client.get_devices().await?;
    let devices_value = serde_json::to_value(result.items)?;
    let count = result.total_record_count.unwrap_or(0) as usize;

    let envelope: CommandOutput =
        OutputEnvelope::success("jellyfin devices list", format!("{} devices", count))
            .with_data(devices_value)
            .with_next_step(NextStep::new(
                "get_device",
                "jellyfin devices get <DEVICE_ID>",
                "Get device details",
            ));

    Ok(envelope)
}

/// Get device details
pub async fn get(device_id: String, profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;
    let device = client.get_device(&device_id).await?;
    let device_value = serde_json::to_value(device)?;

    let envelope: CommandOutput = OutputEnvelope::success(
        "jellyfin devices get",
        format!("Retrieved device: {}", device_id),
    )
    .with_data(device_value);

    Ok(envelope)
}

/// Handle device subcommands
pub async fn handle(action: crate::DeviceCommands, profile: Option<&str>) -> Result<CommandOutput> {
    match action {
        crate::DeviceCommands::List => list(profile).await,
        crate::DeviceCommands::Get { device_id } => get(device_id, profile).await,
    }
}
