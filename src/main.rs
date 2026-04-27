use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    jellyfin_cli::main_entry(std::env::args_os().collect()).await
}
