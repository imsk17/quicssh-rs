use crate::server::ALPN_QUIC_HTTP;
use std::sync::Arc;
use std::{error::Error, net::SocketAddr};

struct SkipServerVerification;

impl SkipServerVerification {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl rustls::client::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item=&[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}

pub(crate) async fn invoke(addr: SocketAddr) -> Result<(), Box<dyn Error>> {
    let mut client_crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();
    client_crypto.alpn_protocols = ALPN_QUIC_HTTP.iter().map(|&x| x.into()).collect();

    let client_config = quinn::ClientConfig::new(Arc::new(client_crypto));
    let mut endpoint = quinn::Endpoint::client("[::]:0".parse().unwrap())?;
    endpoint.set_default_client_config(client_config);
    let conn = endpoint.connect(addr, "localhost").unwrap().await?;


    while let Ok((mut writer, mut reader)) = conn.open_bi().await {
        println!("Established Connection to {}", conn.remote_address());
        let reader_handle = tokio::spawn(async move {
            loop {
                tokio::io::copy(&mut tokio::io::stdin(), &mut writer).await.unwrap_or_default();
            }
        });
        let writer_handle = tokio::spawn(async move {
            loop {
                tokio::io::copy(&mut reader, &mut tokio::io::stdout()).await.unwrap_or_default();
            }
        });
        let _ = tokio::join!(reader_handle, writer_handle);
    }

    Ok(())
}
