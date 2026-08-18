use crate::cli::setup_from_cli;
mod cli;
mod config;
mod dub;
mod file_ops;
// Temporarily out of the equation until I re-inspect it
//mod mix;
mod srt_ops;
mod translate;

#[tokio::main]
async fn main() {
    setup_from_cli().await;
}
