// socketcan = "3.5.0"
#[async_trait]
impl CanTransport for SocketCanTransport {
    pub async fn send(&mut self, frame: CanFrame) -> Result<(), Error> {
        let tx_socket = self
            .tx_sockets
            .get(&channel)
            .ok_or(Error::InvalidChannel(channel))?;

        let can_fd_frame = ();
        self.socket.send(&can_fd_frame).map_err(|error| {
            Error::Transport(format!(
                "failed to send CAN frame on channel {}: {}",
                channel, error
            ))
        })?;

        Ok(())
    }
}
