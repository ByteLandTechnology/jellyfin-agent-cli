//! Item commands

use crate::output::{CommandOutput, NextStep, OutputEnvelope};
use jellyfin_api::{ItemQuery, JellyfinClient};
use jellyfin_core::Result;

/// Get latest items
pub async fn latest(_limit: Option<u32>, profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;

    let result = client.get_latest_items().await?;

    let count = result.len();
    let items = serde_json::to_value(result)?;

    let envelope: CommandOutput =
        OutputEnvelope::success("jellyfin latest", format!("{} latest items", count))
            .with_data(items)
            .with_next_step(NextStep::new(
                "get_details",
                "jellyfin items get <ITEM_ID>",
                "Get detailed information about an item",
            ))
            .with_next_step(NextStep::new(
                "play_item",
                "jellyfin play <ITEM_ID>",
                "Play the media item",
            ));

    Ok(envelope)
}

/// Handle item subcommands
pub async fn handle(action: crate::ItemCommands, profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;

    match action {
        crate::ItemCommands::List {
            parent,
            recursive,
            sort_by,
            limit,
        } => {
            let query = ItemQuery {
                parent_id: parent,
                recursive: Some(recursive),
                sort_by,
                limit: limit.or(Some(50)),
                enable_user_data: Some(true),
                ..Default::default()
            };

            let result = client.get_items(&query).await?;
            let items = serde_json::to_value(result)?;
            let count = items.as_array().map(|a| a.len()).unwrap_or(0);

            let envelope: CommandOutput =
                OutputEnvelope::success("jellyfin items list", format!("{} items", count))
                    .with_data(items)
                    .with_next_step(NextStep::new(
                        "get_details",
                        "jellyfin items get <ITEM_ID>",
                        "Get detailed information about an item",
                    ));

            Ok(envelope)
        }
        crate::ItemCommands::Get { item_id } => {
            let item = client.get_item(&item_id).await?;
            let item_value = serde_json::to_value(item)?;

            let envelope: CommandOutput = OutputEnvelope::success(
                "jellyfin items get",
                format!("Retrieved item: {}", item_id),
            )
            .with_data(item_value.clone())
            .with_next_step(NextStep::new(
                "play_item",
                format!("jellyfin play {}", item_id),
                "Play this media item",
            ))
            .with_next_step(NextStep::new(
                "refresh",
                format!("jellyfin items refresh {}", item_id),
                "Refresh item metadata",
            ));

            Ok(envelope)
        }
        crate::ItemCommands::Refresh { item_id } => {
            client.refresh_item(&item_id).await?;

            let envelope: CommandOutput = OutputEnvelope::success(
                "jellyfin items refresh",
                format!("Refreshed item {}", item_id),
            )
            .with_next_step(NextStep::new(
                "get_details",
                format!("jellyfin items get {}", item_id),
                "Verify the refresh completed",
            ));

            Ok(envelope)
        }
        crate::ItemCommands::Delete { item_id } => {
            client.delete_item(&item_id).await?;

            let envelope: CommandOutput = OutputEnvelope::success(
                "jellyfin items delete",
                format!("Deleted item {}", item_id),
            )
            .with_next_step(NextStep::new(
                "list_items",
                "jellyfin items list",
                "List remaining items",
            ));

            Ok(envelope)
        }
        crate::ItemCommands::Favorite { item_id } => {
            client.add_favorite(&item_id).await?;

            let envelope: CommandOutput = OutputEnvelope::success(
                "jellyfin items favorite",
                format!("Added '{}' to favorites", item_id),
            )
            .with_next_step(NextStep::new(
                "unfavorite",
                format!("jellyfin items unfavorite {}", item_id),
                "Remove from favorites",
            ))
            .with_next_step(NextStep::new(
                "get_details",
                format!("jellyfin items get {}", item_id),
                "View item details",
            ));

            Ok(envelope)
        }
        crate::ItemCommands::Unfavorite { item_id } => {
            client.remove_favorite(&item_id).await?;

            let envelope: CommandOutput = OutputEnvelope::success(
                "jellyfin items unfavorite",
                format!("Removed '{}' from favorites", item_id),
            )
            .with_next_step(NextStep::new(
                "favorite",
                format!("jellyfin items favorite {}", item_id),
                "Add back to favorites",
            ))
            .with_next_step(NextStep::new(
                "get_details",
                format!("jellyfin items get {}", item_id),
                "View item details",
            ));

            Ok(envelope)
        }
        crate::ItemCommands::Favorites => {
            let result = client.get_favorites().await?;
            let items = serde_json::to_value(result)?;
            let count = items.as_array().map(|a| a.len()).unwrap_or(0);

            let envelope: CommandOutput = OutputEnvelope::success(
                "jellyfin items favorites",
                format!("{} favorite items", count),
            )
            .with_data(items)
            .with_next_step(NextStep::new(
                "get_details",
                "jellyfin items get <ITEM_ID>",
                "Get detailed information about an item",
            ));

            Ok(envelope)
        }
        crate::ItemCommands::Rate { item_id, rating } => {
            client.rate_item(&item_id, rating).await?;

            let rating_str = rating
                .map(|r| r.to_string())
                .unwrap_or_else(|| "liked".to_string());
            let envelope: CommandOutput = OutputEnvelope::success(
                "jellyfin items rate",
                format!("Rated item '{}': {}", item_id, rating_str),
            )
            .with_next_step(NextStep::new(
                "get_details",
                format!("jellyfin items get {}", item_id),
                "View item details",
            ));

            Ok(envelope)
        }
    }
}
