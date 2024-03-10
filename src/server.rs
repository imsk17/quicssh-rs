use std::{error::Error, net::SocketAddr};
use std::sync::Arc;

use tokio::net::{TcpStream};

pub const ALPN_QUIC_HTTP: &[&[u8]] = &[b"hq-29"];

pub(crate) async fn invoke(bind: SocketAddr) -> Result<(), Box<dyn Error>> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
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
    eprintln!("listening on {}", endpoint.local_addr()?);
    while let Some(connecting) = endpoint.accept().await {
        println!("Got Connection from {}", connecting.remote_address());
        let mut stream = TcpStream::connect("127.0.0.1:22").await.unwrap();
        tokio::spawn(async move {
            let connection = connecting.await.unwrap();
            if let Ok((mut writer, mut reader)) = connection.accept_bi().await {
                loop {
                    tokio::io::copy(&mut stream, &mut writer).await.unwrap_or_default();
                    tokio::io::copy(&mut reader, &mut stream).await.unwrap_or_default();
                }
            }
        });
    };
    Ok(())
}