use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use socketcan::{
    CanFrame as SocketCanFrame, EmbeddedFrame, ExtendedId, Id, StandardId, tokio::CanSocket,
};

use crate::{error::Error, model::CanFrame};

#[async_trait]
pub trait CanTransport: Send + Sync {
    async fn start(&self) -> Result<(), Error> {
        Ok(())
    }
    async fn send(&self, frame: &CanFrame) -> Result<(), Error>;
    async fn stop(&self) -> Result<(), Error> {
        Ok(())
    }
}

pub trait TransportFactory: Send + Sync {
    fn create(&self, channel: &str) -> Result<Arc<dyn CanTransport>, Error>;
}

#[derive(Clone, Debug)]
pub struct PrintTransport {
    channel: String,
}

#[async_trait]
impl CanTransport for PrintTransport {
    async fn send(&self, frame: &CanFrame) -> Result<(), Error> {
        println!(
            "[{}] id=0x{:x} extended={} data={:02x?}",
            self.channel, frame.id, frame.is_extended, frame.data
        );
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PrintTransportFactory;

impl TransportFactory for PrintTransportFactory {
    fn create(&self, _channel: &str) -> Result<Arc<dyn CanTransport>, Error> {
        Ok(Arc::new(PrintTransport {
            channel: _channel.into(),
        }))
    }
}

pub struct SocketCanTransport {
    socket: CanSocket,
}

impl SocketCanTransport {
    pub fn open(channel: &str) -> Result<Self, Error> {
        Ok(Self {
            socket: CanSocket::open(channel)
                .map_err(|error| Error::Transport(error.to_string()))?,
        })
    }
}

#[async_trait]
impl CanTransport for SocketCanTransport {
    async fn send(&self, frame: &CanFrame) -> Result<(), Error> {
        let id = if frame.is_extended {
            Id::Extended(
                ExtendedId::new(frame.id)
                    .ok_or_else(|| Error::Transport("invalid extended CAN ID".into()))?,
            )
        } else {
            Id::Standard(
                StandardId::new(frame.id as u16)
                    .ok_or_else(|| Error::Transport("invalid standard CAN ID".into()))?,
            )
        };
        let socket_frame = SocketCanFrame::new(id, &frame.data)
            .ok_or_else(|| Error::Transport("invalid classic CAN frame".into()))?;
        self.socket
            .write_frame(socket_frame)
            .await
            .map_err(|error| Error::Transport(error.to_string()))?;
        Ok(())
    }
}

pub struct SocketCanTransportFactory;

impl TransportFactory for SocketCanTransportFactory {
    fn create(&self, channel: &str) -> Result<Arc<dyn CanTransport>, Error> {
        Ok(Arc::new(SocketCanTransport::open(channel)?))
    }
}

#[derive(Default)]
pub enum TransportMode {
    #[default]
    Print,
    SocketCan,
    Custom(Arc<dyn TransportFactory>),
}

impl TransportMode {
    pub fn factory(&self) -> Arc<dyn TransportFactory> {
        match self {
            Self::Print => Arc::new(PrintTransportFactory),
            Self::SocketCan => Arc::new(SocketCanTransportFactory),
            Self::Custom(factory) => factory.clone(),
        }
    }
}

pub const DEFAULT_CYCLE_TIME: Duration = Duration::from_millis(100);
