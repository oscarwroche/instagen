use crate::config;
use axum_server::tls_rustls::RustlsConfig;
use clap::Parser;
use rustls_pemfile::{certs, pkcs8_private_keys};
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use std::{
    fs::File,
    io::{self, BufReader},
    path::{Path, PathBuf},
};

pub fn load_certs(path: &Path) -> io::Result<Vec<CertificateDer<'static>>> {
    certs(&mut BufReader::new(File::open(path)?)).collect()
}

pub fn load_keys(path: &Path) -> io::Result<PrivateKeyDer<'static>> {
    pkcs8_private_keys(&mut BufReader::new(File::open(path)?))
        .next()
        .unwrap()
        .map(Into::into)
}

pub async fn load_server_config() -> RustlsConfig {
    let args = config::Args::parse();
    RustlsConfig::from_pem_file(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(&args.cert_file),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(&args.key_file),
    )
    .await
    .unwrap()
}
