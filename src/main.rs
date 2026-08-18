use crate::cli::setup_from_cli;
mod cli;
mod config;
mod dub;
mod file_ops;
mod mix;
mod srt_ops;
mod translate;

#[tokio::main]
async fn main() {
    setup_from_cli().await;
}
