fn main() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_level(true)
        .init();

    let args: Vec<String> = std::env::args().collect();

    let cfg = match mcp_daddy_core::config_loader::load_config_or_default() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = %e, "failed to load config");
            std::process::exit(1);
        }
    };

    let profile_id = mcp_daddy_core::profile_selection::resolve_profile_id_with_env(&args)
        .unwrap_or_else(|| "default".to_string());
    let profile =
        match mcp_daddy_core::profile_selection::validate_profile_exists(&cfg, &profile_id) {
            Ok(p) => p,
            Err(e) => {
                tracing::error!(profile_id = %profile_id, error = %e, "invalid profile");
                std::process::exit(2);
            }
        };

    tracing::info!(
        app = mcp_daddy_core::APP_NAME,
        version = mcp_daddy_core::build_version(),
        profile_id = %profile_id,
        "starting stdio downstream server"
    );

    let mut server = mcp_daddy_core::downstream_mcp_server::DownstreamMcpServer::new(profile);
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let reader = std::io::BufReader::new(stdin.lock());
    let mut writer = stdout.lock();
    if let Err(e) = server.serve_stdio(reader, &mut writer) {
        tracing::error!(error = %e, "stdio server error");
        std::process::exit(1);
    }
}
