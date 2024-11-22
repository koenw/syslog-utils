use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use native_tls::{TlsAcceptor, TlsStream};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use structopt::StructOpt;

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

fn handle_tcp_client(mut stream: TcpStream) {
    let peer = display_peer!(stream.peer_addr());
    info!("Accepted incoming TCP connection from {peer}");

    let mut buf = [0; 1024];
    loop {
        match stream.read(&mut buf) {
            Ok(len) if len > 0 => {
                debug!("Received {len} (plaintext) bytes from {peer}");
                std::io::stdout().write_all(&buf[..len]).unwrap();
            }
            Ok(_) => {
                debug!("No more bytes left to read");
                info!("TCP Connection with {peer} closed");
                break;
            }
            Err(e) => {
                warn!("Failed to read from {peer}: {e}");
                info!("TCP Connection with {peer} closed");
                break;
            }
        }
    }
}

fn handle_tls_client(mut stream: TlsStream<TcpStream>) {
    let peer = display_peer!(stream.get_ref().peer_addr());
    info!("Accepted incoming TLS connection from {peer}");

    let mut buf = [0; 1024];
    loop {
        match stream.read(&mut buf) {
            Ok(len) if len > 0 => {
                debug!("Received {len} (decrypted) bytes from {peer}");
                std::io::stdout().write_all(&buf[..len]).unwrap();
            }
            Ok(_) => {
                debug!("No more bytes left to read");
                info!("TLS Connection with {peer} closed");
                break;
            }
            Err(e) => {
                warn!("Failed to read from {peer}: {e}");
                info!("TLS Connection with {peer} closed");
                break;
            }
        }
    }
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    pretty_env_logger::formatted_builder()
        .parse_env("SYSLOG_SERVER")
        .filter(None, log::LevelFilter::Info)
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
                thread::spawn(move || {
                    handle_tcp_client(stream);
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
                thread::spawn(move || match acceptor.accept(stream) {
                    Ok(tls_stream) => {
                        handle_tls_client(tls_stream);
                    }
                    Err(e) => error!("Failed to create TLS connection with peer {peer}: {e}"),
                });
            }
        }
    }

    Ok(())
}
