use anyhow::{Context, Result};
use log::{debug, error, info, trace, warn};
use native_tls::TlsAcceptor;
use std::io::Read;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use structopt::StructOpt;
use syslog_loose::{parse_message, Variant};

use utils::identity_from_files;
use utils::Transport;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "syslog-server",
    about = "Simple syslog server for testing purposes",
    setting = structopt::clap::AppSettings::ColoredHelp,
)]
struct Opt {
    /// Syslog Protocol to accept
    #[structopt(possible_values = &Transport::variants(), case_insensitive = true)]
    transport: Transport,

    /// Address to listen on
    #[structopt(long = "address", default_value = "[::]")]
    address: String,

    /// Port to listen on
    #[structopt(long = "port", default_value = "514")]
    port: u16,

    /// Path to file containing TLS private key
    #[structopt(parse(from_os_str))]
    #[structopt(long = "key", required_if("protocol", "tls"))]
    private_key: Option<PathBuf>,

    /// Path to file containing TLS certificate
    #[structopt(parse(from_os_str))]
    #[structopt(long = "cert", required_if("protocol", "tls"))]
    certificate: Option<PathBuf>,
}

macro_rules! display_peer {
    ($peer_addr:expr) => {
        match $peer_addr {
            Ok(ref addr) => format!("{addr}"),
            Err(_) => String::from("unknown peer"),
        }
    };
}

fn handle_client<S: Read>(mut stream: S, peer: String) {
    let mut buf = [0; 16 * 1024];
    loop {
        match stream.read(&mut buf) {
            Ok(len) if len > 0 => {
                let msg_str = String::from_utf8_lossy(&buf[..len]);
                trace!("Received {len} bytes from {peer}: {:?}", msg_str);
                let msg = parse_message(&msg_str, Variant::Either);
                let format = match msg.protocol {
                    syslog_loose::Protocol::RFC3164 => "rfc3164",
                    syslog_loose::Protocol::RFC5424(_) => "rfc5424",
                };
                info!("Received {format} message from {peer}: {msg}");
            }
            Ok(_) => {
                trace!("No more bytes left to read");
                debug!("TCP Connection with {peer} closed");
                break;
            }
            Err(e) => {
                warn!("Failed to read from {peer}: {e}");
                debug!("TCP Connection with {peer} closed");
                break;
            }
        }
    }
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    pretty_env_logger::formatted_builder()
        .filter(None, log::LevelFilter::Info)
        .parse_env("SYSLOG_SERVER_LOG")
        .init();

    match opt.transport {
        Transport::udp => {
            todo!();
        }
        Transport::tcp => {
            let listener = TcpListener::bind(format!("{}:{}", &opt.address, opt.port))
                .with_context(|| format!("Failed to listen on {}:{}", &opt.address, opt.port))?;

            for stream in listener.incoming() {
                let stream = stream?;
                let peer = display_peer!(stream.peer_addr());
                debug!("Accepted incoming TCP connection from {peer}");
                thread::spawn(move || {
                    handle_client(stream, peer);
                });
            }
        }
        Transport::tls => {
            let cert_file = &opt.certificate.unwrap();
            let key_file = &opt.private_key.unwrap();
            let identity = identity_from_files(cert_file, key_file)?;
            let acceptor = Arc::new(TlsAcceptor::new(identity)?);

            let listener = TcpListener::bind(format!("{}:{}", &opt.address, opt.port))
                .with_context(|| format!("Failed to listen on {}:{}", &opt.address, opt.port))?;

            for stream in listener.incoming() {
                let acceptor = acceptor.clone();
                let stream = stream?;

                let peer = display_peer!(stream.peer_addr());
                debug!("Accepted incoming TLS connection from {peer}");
                thread::spawn(move || match acceptor.accept(stream) {
                    Ok(tls_stream) => {
                        handle_client(tls_stream, peer);
                    }
                    Err(e) => error!("Failed to create TLS connection with peer {peer}: {e}"),
                });
            }
        }
    }

    Ok(())
}
