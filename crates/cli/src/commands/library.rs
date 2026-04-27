//! Library commands

use crate::output::{CommandOutput, NextStep, OutputEnvelope};
use jellyfin_api::JellyfinClient;
use jellyfin_core::Result;

/// Add a new media library
pub async fn add(
    name: String,
    collection_type: String,
    paths: Option<Vec<String>>,
    profile: Option<&str>,
) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;
    client
        .add_library(&name, &collection_type, paths.unwrap_or_default())
        .await?;

    let envelope: CommandOutput = OutputEnvelope::success(
        "jellyfin libraries add",
        format!("Added library '{}' ({})", name, collection_type),
    )
    .with_next_step(NextStep::new(
        "list_libraries",
        "jellyfin libraries list",
        "List all libraries",
    ))
    .with_next_step(NextStep::new(
        "browse_library",
        format!("jellyfin libraries items {}", name),
        "Browse items in the new library",
    ));

    Ok(envelope)
}

/// Remove a media library
pub async fn remove(name: String, profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;
    client.remove_library(&name).await?;

    let envelope: CommandOutput = OutputEnvelope::success(
        "jellyfin libraries remove",
        format!("Removed library '{}'", name),
    )
    .with_next_step(NextStep::new(
        "list_libraries",
        "jellyfin libraries list",
        "List remaining libraries",
    ));

    Ok(envelope)
}

/// Handle library subcommands
pub async fn handle(
    action: crate::LibraryCommands,
    profile: Option<&str>,
) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;

    match action {
        crate::LibraryCommands::List => {
            let libraries = client.get_libraries().await?;
            let libs_value = serde_json::to_value(libraries)?;
            let count = libs_value.as_array().map(|a| a.len()).unwrap_or(0);

            let envelope: CommandOutput =
                OutputEnvelope::success("jellyfin libraries list", format!("{} libraries", count))
                    .with_data(libs_value)
                    .with_next_step(NextStep::new(
                        "browse_library",
                        "jellyfin libraries items <LIBRARY>",
                        "Browse items in a library",
                    ));

            Ok(envelope)
        }
        crate::LibraryCommands::Items { library, limit: _ } => {
            // First find the library by name or ID
            let libraries = client.get_libraries().await?;

            let library_id = if let Some(lib) = libraries.iter().find(|l| l.name == library) {
                lib.id.clone()
            } else {
                // Assume it's an ID
                library.clone()
            };

            let result = client.get_library_items(&library_id).await?;
            let items_value = serde_json::to_value(result)?;
            let count = items_value.as_array().map(|a| a.len()).unwrap_or(0);

            let envelope: CommandOutput = OutputEnvelope::success(
                "jellyfin libraries items",
                format!("{} items in library '{}'", count, library_id),
            )
            .with_data(items_value)
            .with_next_step(NextStep::new(
                "get_details",
                "jellyfin items get <ITEM_ID>",
                "Get detailed information about an item",
            ))
            .with_next_step(NextStep::new(
                "refresh_library",
                format!("jellyfin items refresh {}", library_id),
                "Refresh library metadata",
            ));

            Ok(envelope)
        }
        crate::LibraryCommands::Add {
            name,
            collection_type,
            paths,
        } => add(name, collection_type, paths, profile).await,
        crate::LibraryCommands::Remove { name } => remove(name, profile).await,
    }
}
