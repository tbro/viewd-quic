use anyhow::Result;
use s2n_quic::{
    client::Connect,
    stream::{ReceiveStream, SendStream}, Client
};
use std::net::SocketAddr;
use tracing::debug;

/// NOTE: this certificate is to be used for demonstration purposes only!
pub static CERT_PEM: &str = include_str!(concat!("../../tls/cert.pem"));

pub struct QuicService {
    client: Client,
    connect: Connect,
}

impl QuicService {
    pub fn new(host: String) -> Result<QuicService> {
        debug! {"\n{}", CERT_PEM};
        let client = Client::builder()
            .with_tls(CERT_PEM)?
            .with_io("0.0.0.0:0")?
            .start()?;

        let remote: SocketAddr = host.parse()?;
        let connect = Connect::new(remote).with_server_name("localhost");
        let q = QuicService { client, connect };
        Ok(q)
    }
    pub async fn connect(self) -> Result<(ReceiveStream, SendStream)> {
        let mut connection = self.client.connect(self.connect).await?;

        // ensure the connection doesn't time out with inactivity
        connection.keep_alive(true)?;

        let stream = connection.open_bidirectional_stream().await?;
        Ok(stream.split())
    }
}
