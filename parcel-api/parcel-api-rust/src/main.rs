use std::sync::Arc;

use parcel_api::{app::AppState, config::AppConfig, service::StubParcelService};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() {
    init_tracing();

    let config = AppConfig::load();
    tracing::info!(profile = ?std::env::var("APP_PROFILE").ok(), "Loaded config");

    let state = Arc::new(AppState {
        parcels: StubParcelService::load(&config.parcel_data_dir),
        config: config.clone(),
    });

    let app = parcel_api::build_app(state);
    let addr = format!("0.0.0.0:{}", config.server_port);
    tracing::info!(%addr, "parcel-api starting");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("bind tcp listener");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
}

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let json_layer = tracing_subscriber::fmt::layer().json();
    tracing_subscriber::registry()
        .with(env_filter)
        .with(json_layer)
        .init();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.ok();
    };
    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut sig) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            sig.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("shutdown signal received");
}
