//! Scheduled tasks commands

use crate::output::{CommandOutput, NextStep, OutputEnvelope};
use jellyfin_api::JellyfinClient;
use jellyfin_core::Result;

/// List all scheduled tasks
pub async fn list(profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;
    let tasks = client.get_scheduled_tasks().await?;
    let tasks_value = serde_json::to_value(tasks)?;
    let count = tasks_value.as_array().map(|a| a.len()).unwrap_or(0);

    let envelope: CommandOutput =
        OutputEnvelope::success("jellyfin scheduled-tasks list", format!("{} tasks", count))
            .with_data(tasks_value)
            .with_next_step(NextStep::new(
                "get_task",
                "jellyfin scheduled-tasks get <TASK_ID>",
                "Get task details",
            ));

    Ok(envelope)
}

/// Get task details
pub async fn get(task_id: String, profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;
    let task = client.get_scheduled_task(&task_id).await?;
    let task_value = serde_json::to_value(task)?;

    let envelope: CommandOutput = OutputEnvelope::success(
        "jellyfin scheduled-tasks get",
        format!("Retrieved task: {}", task_id),
    )
    .with_data(task_value)
    .with_next_step(NextStep::new(
        "start_task",
        format!("jellyfin scheduled-tasks start {}", task_id),
        "Start this task",
    ));

    Ok(envelope)
}

/// Start a scheduled task
pub async fn start(task_id: String, profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;
    client.start_scheduled_task(&task_id).await?;

    let envelope: CommandOutput = OutputEnvelope::success(
        "jellyfin scheduled-tasks start",
        format!("Started task: {}", task_id),
    )
    .with_next_step(NextStep::new(
        "stop_task",
        format!("jellyfin scheduled-tasks stop {}", task_id),
        "Stop this task",
    ))
    .with_next_step(NextStep::new(
        "list_tasks",
        "jellyfin scheduled-tasks list",
        "List all tasks",
    ));

    Ok(envelope)
}

/// Stop a scheduled task
pub async fn stop(task_id: String, profile: Option<&str>) -> Result<CommandOutput> {
    let client = JellyfinClient::from_config(profile).await?;
    client.stop_scheduled_task(&task_id).await?;

    let envelope: CommandOutput = OutputEnvelope::success(
        "jellyfin scheduled-tasks stop",
        format!("Stopped task: {}", task_id),
    )
    .with_next_step(NextStep::new(
        "start_task",
        format!("jellyfin scheduled-tasks start {}", task_id),
        "Start this task again",
    ))
    .with_next_step(NextStep::new(
        "list_tasks",
        "jellyfin scheduled-tasks list",
        "List all tasks",
    ));

    Ok(envelope)
}

/// Handle scheduled task subcommands
pub async fn handle(
    action: crate::ScheduledTaskCommands,
    profile: Option<&str>,
) -> Result<CommandOutput> {
    match action {
        crate::ScheduledTaskCommands::List => list(profile).await,
        crate::ScheduledTaskCommands::Get { task_id } => get(task_id, profile).await,
        crate::ScheduledTaskCommands::Start { task_id } => start(task_id, profile).await,
        crate::ScheduledTaskCommands::Stop { task_id } => stop(task_id, profile).await,
    }
}
