use std::{net::SocketAddr, sync::Arc, path::Path};

use axum_can::{
    axum_can::{AppState, serve},
    can_sender::MockHardware,
    registry::Registry,
    loader::setup_registry,
};
use tracing::{error, info, level_filters::LevelFilter};

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_file(false)
        .with_line_number(true)
        .with_thread_ids(false)
        .with_target(true)
        .with_max_level(LevelFilter::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let mut registry = Registry::default();
    let hardware = MockHardware;
    let config_path = Path::new("can_signal.json");

    if let Err(e) = setup_registry(&mut registry, &hardware, config_path).await {
        error!("Init CAN registry failed: {}, exit the process", e);
        std::process::exit(1);
    }

    let state = Arc::new(AppState {
        registry: Arc::new(registry),
    });

    let port = 8028;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Listening on localhost:{}", port);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    serve(listener, state).await
}
