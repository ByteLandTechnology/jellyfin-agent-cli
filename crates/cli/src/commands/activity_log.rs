//! Activity log commands

use crate::output::{CommandOutput, OutputEnvelope};
use jellyfin_api::JellyfinClient;
use jellyfin_core::Result;

/// Get activity log entries
pub async fn entries(profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;
    let result = client.get_activity_log().await?;
    let value = serde_json::to_value(&result.items)?;
    let count = result.items.len();

    let envelope: CommandOutput = OutputEnvelope::success(
        "jellyfin activity-log",
        format!("{} activity entries", count),
    )
    .with_data(value);

    Ok(envelope)
}
