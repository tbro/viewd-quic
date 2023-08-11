use anyhow::{anyhow, bail, Result};
use bytes::Bytes;
use s2n_quic::{stream::BidirectionalStream, Connection};
use tracing::{debug, error, info};
use crossbeam_channel::{Sender, Receiver};

/// spawn tasks to accept connection, connect to stream and pass
/// input down the wire
pub async fn handle_connection(tx: Sender<Bytes>, rx: Receiver<Bytes>, mut connection: Connection) -> Result<()> {
    loop {
        let stream = match connection.accept_bidirectional_stream().await {
            Ok(Some(stream)) => stream,
            Ok(None) => {
                info! {"connection closed"};
                return Ok(());
            }
            Err(e) => bail!("{}", e.to_string()),
        };
        let fut = handle_request(tx.clone(), rx.clone(), stream);
        tokio::spawn(async move {
            if let Err(e) = fut.await {
                error!("failed: {reason}", reason = e.to_string());
            }
        });
    }
}
/// Sends request to controler and waits for response. Forwards response to client
pub async fn handle_request(tx: Sender<Bytes>, rx: Receiver<Bytes>, mut stream: BidirectionalStream) -> Result<()> {
    loop {
        match stream.receive().await {
            Ok(Some(request)) => {
                tx.send(request)
                    .map_err(|e| anyhow!("control reciever closed: {}", e))?;

		// block to wait for response
		// send it to the client
		if let Ok(response) = rx.recv() {
                    stream
			.send(response)
			.await
			.map_err(|e| anyhow!("channel empty: {}", e))?;
		}
            }
            Ok(None) => debug!("stream finished"),
            Err(e) => bail!("{}", e.to_string()),
        }
    }
}
