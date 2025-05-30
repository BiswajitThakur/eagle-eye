use std::path::Path;

use rustls::{
    ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
};

pub(crate) fn server_config() -> ServerConfig {
    let key = load_key("key/key.pem");
    let cert = load_cert("key/cert.pem");
    ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert, key)
        .expect("bad certificates or key")
}

fn load_key<P: AsRef<Path>>(path: P) -> PrivateKeyDer<'static> {
    PrivateKeyDer::from_pem_file(path).unwrap()
}

fn load_cert<P: AsRef<Path>>(path: P) -> Vec<CertificateDer<'static>> {
    let certs: Vec<_> = CertificateDer::pem_file_iter(path)
        .unwrap()
        .map(|v| v.unwrap())
        .collect();
    certs
}
