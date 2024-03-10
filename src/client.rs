use crate::server::ALPN_QUIC_HTTP;
use std::sync::Arc;
use std::{error::Error, net::SocketAddr};
use quinn::VarInt;
use tokio::io::{stdin, stdout};
use crate::verifier::SkipServerVerification;

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
        let mut server = tokio::io::join(&mut reader, &mut writer);
        let (inp, out) = (stdin(), stdout());
        let mut localhost = tokio::io::join(inp, out);
        if let Ok((read, wrote)) = tokio::io::copy_bidirectional(&mut server, &mut localhost).await {
            println!("Read {read}bytes. Wrote: {wrote}bytes");
            continue;
        };
        break;
    }
    conn.close(VarInt::from_u32(0), b"Closing Connection");
    Ok(())
}
