use anyhow::{Context, Result};
use native_tls::Identity;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use structopt::clap::arg_enum;

arg_enum! {
    #[derive(Debug)]
    #[allow(non_camel_case_types)]
    pub enum Transport {
        tcp,
        udp,
        tls,
    }
}

arg_enum! {
    #[derive(Debug)]
    #[allow(non_camel_case_types)]
    pub enum Severity {
        emergency,
        alert,
        critical,
        error,
        warning,
        notice,
        informational,
        debug,
    }
}

arg_enum! {
    #[derive(Debug)]
    #[allow(non_camel_case_types)]
    pub enum Format {
        rfc3164,
        rfc5424,
    }
}

impl From<Severity> for fasyslog::Severity {
    fn from(val: Severity) -> Self {
        match val {
            Severity::emergency => fasyslog::Severity::EMERGENCY,
            Severity::alert => fasyslog::Severity::ALERT,
            Severity::critical => fasyslog::Severity::CRITICAL,
            Severity::error => fasyslog::Severity::ERROR,
            Severity::warning => fasyslog::Severity::WARNING,
            Severity::notice => fasyslog::Severity::NOTICE,
            Severity::informational => fasyslog::Severity::INFORMATIONAL,
            Severity::debug => fasyslog::Severity::DEBUG,
        }
    }
}

pub fn identity_from_files<C: AsRef<Path>, K: AsRef<Path>>(
    certificate: C,
    key: K,
) -> Result<Identity> {
    let cert_path = certificate.as_ref();
    let key_path = key.as_ref();

    let mut cert_file = File::open(cert_path)
        .with_context(|| format!("Failed to open certificate {}", cert_path.display()))?;
    let mut cert = vec![];
    cert_file
        .read_to_end(&mut cert)
        .with_context(|| format!("Failed to read certificate {}", cert_path.display()))?;

    let mut key_file = File::open(key_path)
        .with_context(|| format!("Failed to open private key {}", key_path.display()))?;
    let mut key = vec![];
    key_file
        .read_to_end(&mut key)
        .with_context(|| format!("Failed to read private key {}", &key_path.display()))?;

    let identity = Identity::from_pkcs8(&cert, &key).with_context(|| {
        format!(
            "Failed to construct identity from certificate {} and key {}",
            cert_path.display(),
            key_path.display()
        )
    })?;

    Ok(identity)
}
