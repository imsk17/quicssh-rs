use std::sync::Arc;
use std::{error::Error, net::SocketAddr};
use tokio::net::TcpStream;
use tracing::{debug, info};

pub const ALPN_QUIC_HTTP: &[&[u8]] = &[b"hq-29"];

pub(crate) async fn invoke(bind: SocketAddr) -> Result<(), Box<dyn Error>> {
    let cert = rcgen::generate_simple_self_signed(vec!["bruh".into()]).unwrap();
    let key = cert.serialize_private_key_der();
    let cert = cert.serialize_der().unwrap();
    let key = rustls::PrivateKey(key);
    let certs = vec![rustls::Certificate(cert)];

    let mut server_crypto = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;
    server_crypto.alpn_protocols = ALPN_QUIC_HTTP.iter().map(|&x| x.into()).collect();
    server_crypto.key_log = Arc::new(rustls::KeyLogFile::new());

    let mut server_config = quinn::ServerConfig::with_crypto(Arc::new(server_crypto));
    let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
    transport_config.max_concurrent_uni_streams(0_u8.into());
    server_config.use_retry(true);

    let endpoint = quinn::Endpoint::server(server_config, bind)?;
    info!("Listening for connections on {}", endpoint.local_addr()?);
    while let Some(connecting) = endpoint.accept().await {
        info!("Got Connection from {}", connecting.remote_address());
        tokio::spawn(async move {
            let connection = connecting.await.unwrap();
            while let Ok((mut writer, mut reader)) = connection.accept_bi().await {
                let mut stream = TcpStream::connect("127.0.0.1:22")
                    .await
                    .expect("Failed to connect to sshd");
                tokio::spawn(async move {
                    loop {
                        let mut joined = tokio::io::join(&mut reader, &mut writer);
                        if let Ok((read, wrote)) =
                            tokio::io::copy_bidirectional(&mut stream, &mut joined).await
                        {
                            debug!("Read from SSH: {read}bytes. Wrote to SSH: {wrote}bytes.");
                            continue;
                        };
                        break;
                    }
                });
            }
        });
    }
    Ok(())
}
