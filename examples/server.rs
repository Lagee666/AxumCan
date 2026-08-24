use std::{net::SocketAddr, sync::Arc};

use axum_can::{
    axum_can::{AppState, serve},
    registry::Registry,
    transport::TransportMode,
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
    // Registry::default() loads channel names and messages from can_signal.json.
    // Each channel name is passed directly to the selected transport.
    registry.set_transport_mode(TransportMode::Print);
    if let Err(error) = registry.init().await {
        error!(%error, "failed to initialize CAN registry");
        return;
    }

    let state = Arc::new(AppState {
        registry: Arc::new(registry),
    });
    let addr = SocketAddr::from(([0, 0, 0, 0], 8028));
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(error) => {
            error!(%error, "failed to bind HTTP listener");
            return;
        }
    };
    info!(address = %addr, "AxumCan dashboard is listening");
    serve(listener, state).await;
}
