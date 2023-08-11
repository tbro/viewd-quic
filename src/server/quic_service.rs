use anyhow::Result;
use bytes::Bytes;
use crossbeam_channel::{Receiver, Sender};
use s2n_quic::Server;
use tracing::{error, info};

use crate::server::handlers::handle_connection;

/// NOTE: this certificate is to be used for demonstration purposes only!
pub static CERT_PEM: &str = include_str!(concat!("../../tls/cert.pem"));
/// NOTE: this certificate is to be used for demonstration purposes only!
pub static KEY_PEM: &str = include_str!(concat!("../../tls/key.pem"));

#[derive(Debug)]
/// Server side of Quic connection
pub struct QuicService {
    server: Server,
    tx_req: Sender<Bytes>,
    rx_res: Receiver<Bytes>
}

impl QuicService {
    pub fn new(
        bind: String,
	tx_req: Sender<Bytes>, rx_res: Receiver<Bytes>
    ) -> Result<QuicService> {
        info! {"\n{}", CERT_PEM};
        let server = Server::builder()
            .with_tls((CERT_PEM, KEY_PEM))?
            .with_io(bind.as_str())?
            .start()?;
        let server = QuicService {
            server,
	    tx_req,
	    rx_res,
        };
        Ok(server)
    }

    pub fn listen_task(mut self) {
        tokio::spawn(async move {
            while let Some(connection) = self.server.accept().await {
                info! {
                    "new connection ({}): {}",
                    connection.id(),
                    connection.remote_addr().unwrap()
                };
                let fut = handle_connection(self.tx_req.clone(), self.rx_res.clone(), connection);
                tokio::spawn(async move {
                    if let Err(e) = fut.await {
                        error!("connection failed: {reason}", reason = e.to_string())
                    }
                });
            }
        });
    }
}
