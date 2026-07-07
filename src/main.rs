use std::{net::SocketAddr, sync::Arc};

use axum_can::{
    axum_can::{AppState, serve},
    can_sender::MockSocket,
    registry::Registry,
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
    registry.set_socket(Box::new(MockSocket));
    if let Err(e) = registry.init().await {
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
