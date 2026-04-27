//! Remote search commands

use crate::output::{CommandOutput, NextStep, OutputEnvelope};
use jellyfin_api::{JellyfinClient, RemoteSearchInfo, RemoteSearchQuery};
use jellyfin_core::Result;

/// Search remote providers
pub async fn search(
    query: String,
    item_type: Option<String>,
    year: Option<u32>,
    profile: Option<&str>,
) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;

    let search_info = RemoteSearchInfo {
        name: Some(query.clone()),
        year,
        provider_ids: None,
    };

    let remote_query = RemoteSearchQuery {
        search_info,
        item_type: item_type.clone(),
        metadata_country_code: None,
        metadata_language_code: None,
    };

    let results = client.remote_search(&remote_query).await?;
    let value = serde_json::to_value(results)?;
    let count = value.as_array().map(|a| a.len()).unwrap_or(0);

    let item_type_str = item_type.unwrap_or_else(|| "media".to_string());
    let envelope: CommandOutput = OutputEnvelope::success(
        "jellyfin remote-search",
        format!(
            "Found {} results for '{}' ({})",
            count, query, item_type_str
        ),
    )
    .with_data(value)
    .with_next_step(NextStep::new(
        "add_to_library",
        "jellyfin items add",
        "Add item to library",
    ));

    Ok(envelope)
}
