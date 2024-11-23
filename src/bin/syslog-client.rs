use anyhow::{Context, Result};
use fasyslog::{sender, SDElement};
use std::collections::HashMap;
use std::io::{self, BufRead};
use structopt::StructOpt;

use utils::{Format, Severity, Transport};

#[derive(Debug, Default)]
struct SDElements(HashMap<String, String>);

impl std::str::FromStr for SDElements {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hm: HashMap<String, String> = s
            .split(',')
            .filter_map(|kv| {
                kv.split_once('=')
                    .map(|(k, v)| (k.to_owned(), v.to_owned()))
            })
            .collect();
        Ok(SDElements(hm))
    }
}

/// Simple Syslog client
#[derive(Debug, StructOpt)]
#[structopt(
    name = "syslog-client",
    about = "Syslog client for diagnostic purposes",
    setting =  structopt::clap::AppSettings::ColoredHelp,
)]
struct Opt {
    /// Syslog transport protocol
    #[structopt(possible_values = &Transport::variants(), case_insensitive = true)]
    transport: Transport,

    /// Syslog server host
    #[structopt(long, default_value = "localhost")]
    host: String,

    /// Syslog server port [default: 514(UDP), 601(TCP) or 6514(TLS)]
    #[structopt(long)]
    port: Option<u16>,

    /// Syslog server TLS domain [default: syslog server host]
    #[structopt(long)]
    tls_domain: Option<String>,

    /// Accept invalid TLS certificates (insecure!)
    #[structopt(long)]
    accept_invalid_certs: bool,

    /// Accept TLS certificates for invalid hostnames (insecure!)
    #[structopt(long)]
    accept_invalid_hostnames: bool,

    /// Message format
    #[structopt(
        long,
        default_value = "rfc3164",
        case_insensitive = true,
        help = "Either rfc5424 or rfc3164"
    )]
    format: Format,

    /// RFC5424 SD-ID (Structured Data ID)
    #[structopt(long, default_value = "syslog-client@1234")]
    sd_id: String,

    /// RFC5424 message id, e.g. "TCPIN"
    #[structopt(long)]
    msg_id: Option<String>,

    /// RFC5424 Structured Data Elements, e.g. "key=value,anotherkey=anothervalue"
    #[structopt(long, required_if("format", "rfc5424"))]
    sd_elements: Option<SDElements>,

    /// Message severity
    #[structopt(long, default_value = "notice", case_insensitive = true)]
    severity: Severity,

    /// Read log messages from stdin
    #[structopt(long)]
    stdin: bool,

    /// Log messages to send
    messages: Vec<String>,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    let severity = opt.severity.into();

    let mut sender = match opt.transport {
        Transport::udp => sender::SyslogSender::Udp(sender::udp(
            "0.0.0.0:0",
            (opt.host.clone(), opt.port.unwrap_or(514)),
        )?),
        Transport::tcp => sender::SyslogSender::Tcp(
            sender::tcp((opt.host, opt.port.unwrap_or(601)))
                .context("Failed to send message over TCP")?,
        ),
        Transport::tls => {
            let mut connector = native_tls::TlsConnector::builder();
            let connector = connector.danger_accept_invalid_certs(opt.accept_invalid_certs);
            let connector = connector.danger_accept_invalid_hostnames(opt.accept_invalid_hostnames);
            sender::SyslogSender::NativeTlsSender(
                sender::native_tls_with(
                    (opt.host.clone(), opt.port.unwrap_or(6514)),
                    opt.tls_domain.unwrap_or(opt.host),
                    connector,
                )
                .context("Failed to send message over TLS")?,
            )
        }
    };

    let sd_elements = opt.sd_elements.unwrap_or(SDElements::default());
    let stdin_messages = opt
        .stdin
        .then_some(io::stdin().lock().lines().map_while(|line| line.ok()))
        .into_iter()
        .flatten();
    let iter = opt.messages.into_iter().chain(stdin_messages);

    for line in iter {
        if let Err(e) = match &opt.format {
            Format::rfc3164 => sender.send_rfc3164(severity, line),
            Format::rfc5424 => {
                let mut elements = match SDElement::new(&opt.sd_id) {
                    Ok(v) => v,
                    Err(_) => {
                        eprintln!("Failed to contruct Structured Data Element. Perhaps you passed an invalid SD ID?");
                        break;
                    }
                };
                for (key, value) in &sd_elements.0 {
                    if let Err(e) = elements.add_param(key, value) {
                        eprintln!("Failed to parameter {key} to {value}: {e}");
                    };
                }
                sender.send_rfc5424(severity, opt.msg_id.clone(), vec![elements], line)
            }
        } {
            eprintln!("Failed to send message: {e}");
        }

        if let Err(e) = sender.flush() {
            eprintln!("Failed to flush message: {e}");
        }
    }

    Ok(())
}
