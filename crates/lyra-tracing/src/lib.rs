use tracing_subscriber::{EnvFilter, filter::LevelFilter};

pub fn init() {
    tracing_subscriber::fmt()
        .with_env_filter(build_env_filter())
        .init();
}

fn build_env_filter() -> EnvFilter {
    if std::env::var_os(EnvFilter::DEFAULT_ENV).is_some() {
        return EnvFilter::from_default_env();
    }

    // Keep Lyra targets chatty by default without dragging noisy third-party
    // dependencies like ort up to INFO when RUST_LOG is unset.
    EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .parse_lossy("lyra=info")
}
