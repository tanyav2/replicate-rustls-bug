use log::*;
use serde::Deserialize;
use std::fs;
use std::io::{self, BufReader};
use std::sync::Arc;
use tokio::io::{copy, split, stdin as tokio_stdin, stdout as tokio_stdout, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::rustls;

#[derive(Debug, PartialEq, Deserialize)]
pub struct ClientConf {
    pub addr: String,
    pub ca_cert: String,
    pub client_cert: String,
    pub client_key: String,
    pub connect: String,
}

fn load_conf(path: &str) -> Result<ClientConf, serde_yaml::Error> {
    let contents = fs::read_to_string(path).unwrap();
    let client_conf: ClientConf = serde_yaml::from_str(&contents)?;
    Ok(client_conf)
}

fn load_certs(filename: &str) -> Vec<rustls::Certificate> {
    let certfile = fs::File::open(filename).expect("cannot open certificate file");
    let mut reader = BufReader::new(certfile);
    rustls_pemfile::certs(&mut reader)
        .unwrap()
        .iter()
        .map(|v| rustls::Certificate(v.clone()))
        .collect()
}

fn load_cert_store(filename: &str) -> rustls::RootCertStore {
    let mut trust_store = rustls::RootCertStore::empty();
    for cert in &load_certs(filename) {
        trust_store
            .add(cert)
            .expect("failed to add certificate to trust store");
    }
    trust_store
}

fn load_private_key(filename: &str) -> rustls::PrivateKey {
    let keyfile = fs::File::open(filename).expect("cannot open private key file");
    let mut reader = BufReader::new(keyfile);
    loop {
        match rustls_pemfile::read_one(&mut reader).expect("cannot parse private key .pem file") {
            Some(rustls_pemfile::Item::RSAKey(key)) => return rustls::PrivateKey(key),
            Some(rustls_pemfile::Item::PKCS8Key(key)) => return rustls::PrivateKey(key),
            None => break,
            _ => {}
        }
    }
    panic!("no keys found",);
}

#[tokio::main]
async fn main() -> io::Result<()> {
    env_logger::builder().init();
    let args: Vec<String> = std::env::args().collect();
    let conf = load_conf(&args[1]).unwrap();

    let mut tls_client_config = rustls::ClientConfig::new();
    tls_client_config
        .set_single_client_cert(
            load_certs(&conf.client_cert),
            load_private_key(&conf.client_key),
        )
        .expect("failed to configure client certificate");
    tls_client_config.root_store = load_cert_store(&conf.ca_cert);
    let tls_client = tokio_rustls::TlsConnector::from(Arc::new(tls_client_config));
    debug!("Starting client on {}", &conf.addr);

    loop {
        let tls_client = tls_client.clone();
        let stream = TcpStream::connect(&conf.connect).await?;
        let domain = webpki::DNSNameRef::try_from_ascii_str("localhost").unwrap();
        let mut stream = tls_client.connect(domain, stream).await.map_err(|e| {
            error!("failed to execute request with err: {:?}", e);
            e
        })?;
        let content = format!("GET / HTTP/1.0\r\nHost: {}\r\n\r\n", "localhost");
        stream.write_all(content.as_bytes()).await?;
        let (mut reader, mut writer) = split(stream);
        let (mut stdin, mut stdout) = (tokio_stdin(), tokio_stdout());

        tokio::select! {
            ret = copy(&mut reader, &mut stdout) => {
                ret?;
            },
            ret = copy(&mut stdin, &mut writer) => {
                ret?;
                writer.shutdown().await?
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(15000)).await;
    }
}
