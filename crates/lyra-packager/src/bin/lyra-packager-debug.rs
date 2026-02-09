use anyhow::Result;
use lyra_packager::{
    build_packager_state_with_defaults, parse_single_input_path_arg, serve_packager,
};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let input = parse_single_input_path_arg()?;
    let state = build_packager_state_with_defaults(&input)?;
    serve_packager(state, "0.0.0.0:4422").await
}
