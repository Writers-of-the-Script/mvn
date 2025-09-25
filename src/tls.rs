use std::sync::Arc;

use rustls::{
    client::{danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier}, WantsClientCert}, crypto::WebPkiSupportedAlgorithms, pki_types::{CertificateDer, ServerName, UnixTime}, server::ParsedCertificate, ClientConfig, ConfigBuilder, DigitallySignedStruct, Error as TLSError, RootCertStore, SignatureScheme, WantsVerifier
};

#[derive(Debug)]
struct NoVerifier;

impl ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer,
        _intermediates: &[CertificateDer],
        _server_name: &ServerName,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, TLSError> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, TLSError> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, TLSError> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::RSA_PKCS1_SHA1,
            SignatureScheme::ECDSA_SHA1_Legacy,
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
            SignatureScheme::ECDSA_NISTP521_SHA512,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
            SignatureScheme::ED25519,
            SignatureScheme::ED448,
        ]
    }
}

#[derive(Debug)]
struct IgnoreHostname {
    roots: RootCertStore,
    signature_algorithms: WebPkiSupportedAlgorithms,
}

impl IgnoreHostname {
    pub fn new(roots: RootCertStore, signature_algorithms: WebPkiSupportedAlgorithms) -> Self {
        Self {
            roots,
            signature_algorithms,
        }
    }
}

impl ServerCertVerifier for IgnoreHostname {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        now: UnixTime,
    ) -> Result<ServerCertVerified, TLSError> {
        let cert = ParsedCertificate::try_from(end_entity)?;

        rustls::client::verify_server_cert_signed_by_trust_anchor(
            &cert,
            &self.roots,
            intermediates,
            now,
            self.signature_algorithms.all,
        )?;
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, TLSError> {
        rustls::crypto::verify_tls12_signature(message, cert, dss, &self.signature_algorithms)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, TLSError> {
        rustls::crypto::verify_tls13_signature(message, cert, dss, &self.signature_algorithms)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.signature_algorithms.supported_schemes()
    }
}

fn roots() -> Result<RootCertStore, TLSError> {
    let mut roots = RootCertStore::empty();

    #[cfg(all(
        unix,
        not(target_os = "android"),
        not(target_vendor = "apple"),
        not(target_arch = "wasm32"),
    ))]
    {
        let result = rustls_native_certs::load_native_certs();
        let (added, ignored) = roots.add_parsable_certificates(result.certs);
        if ignored > 0 {
            warn!("{ignored} platform CA root certificates were ignored due to errors");
        }

        for error in result.errors {
            warn!("Error loading CA root certificate: {error}");
        }

        // Don't return an error if this fails when other roots have already been loaded via
        // `new_with_extra_roots`. It leads to extra failure cases where connections would otherwise still work.
        if roots.is_empty() {
            return Err(rustls::Error::General(
                "No CA certificates were loaded from the system".to_owned(),
            ));
        } else {
            debug!("Loaded {added} CA root certificates from the system");
        }
    }

    Ok(roots)
}

pub trait CustomVerifiers {
    fn with_no_verifier(self) -> Result<ConfigBuilder<ClientConfig, WantsClientCert>, TLSError>;

    fn with_ignore_hosts_verifier(
        self,
    ) -> Result<ConfigBuilder<ClientConfig, WantsClientCert>, TLSError>;
}

impl CustomVerifiers for ConfigBuilder<ClientConfig, WantsVerifier> {
    fn with_no_verifier(self) -> Result<ConfigBuilder<ClientConfig, WantsClientCert>, TLSError> {
        Ok(self
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoVerifier)))
    }

    fn with_ignore_hosts_verifier(
        self,
    ) -> Result<ConfigBuilder<ClientConfig, WantsClientCert>, TLSError> {
        let it = IgnoreHostname::new(
            roots()?,
            self.crypto_provider().signature_verification_algorithms,
        );

        Ok(self
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(it)))
    }
}
