fn main() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_level(true)
        .init();

    tracing::info!(
        app = mcp_daddy_core::APP_NAME,
        version = mcp_daddy_core::build_version(),
        "starting"
    );

    println!(
        "{} v{}",
        mcp_daddy_core::APP_NAME,
        mcp_daddy_core::build_version()
    );
}
