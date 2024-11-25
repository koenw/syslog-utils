# syslog-utils

Simple syslog RFC3164 and RFC5424 command-line client & server.

The client supports UDP, TCP and TLS transports and is useful to test syslog
servers or simply to send syslog messages. The server supports TCP and TLS and
will simply print any parsed messages to stdout.


## Getting Started

The easiest way to get started is by using the statically compiled release
binaries or the docker image. Download the *syslog-client* and *syslog-server*
binaries from the [releases](https://github.com/koenw/syslog-utils/releases)
page.

Run the client with `docker run ghcr.io/koenw/syslog-utils syslog-client` or
the server with `docker run ghcr.io/koenw/syslog-utils syslog-server`.


## Usage


### syslog-client

Simple command line syslog client to send RFC5424 or RFC3164 messages over UDP,
TCP or TLS.


#### Help

```sh
❯ syslog-client --help
syslog-client 0.1.0
Syslog client for diagnostic purposes

USAGE:
    syslog-client [FLAGS] [OPTIONS] <transport> [messages]...

FLAGS:
        --accept-invalid-certs        Accept invalid TLS certificates (insecure!)
        --accept-invalid-hostnames    Accept TLS certificates for invalid hostnames (insecure!)
    -h, --help                        Prints help information
        --stdin                       Read log messages from stdin
    -V, --version                     Prints version information

OPTIONS:
        --format <format>              Either rfc5424 or rfc3164 [default: rfc3164]
        --host <host>                  Syslog server host [default: localhost]
        --msg-id <msg-id>              RFC5424 message id, e.g. "TCPIN"
        --port <port>
            Syslog server port [default: 514(UDP), 601(TCP) or 6514(TLS)]

        --sd-elements <sd-elements>
            RFC5424 Structured Data Elements, e.g. "key=value,anotherkey=anothervalue"

        --sd-id <sd-id>
            RFC5424 SD-ID (Structured Data ID) [default: syslog-client@1234]

        --severity <severity>          Message severity [default: notice]
        --tls-domain <tls-domain>      Syslog server TLS domain [default: syslog server host]

ARGS:
    <transport>      Syslog transport protocol [possible values: tcp, udp,
                     tls]
    <messages>...    Log messages to send
```


#### Examples

| Command | Description |
| --- | --- |
| `syslog-client tcp --port 5014 --format rfc5424 --sd-elements="key=value,nothing=equal" hello` | Send the message "hello" to port 5014, with RFC5424 Structured Data |
| `syslog-client tls --host syslog.example.com --stdin` | Read messages on stdin and send them over TLS to syslog.example.com |
| `syslog-client tls --accept-invalid-certs hello` | Send messages over TLS, ignoring certificate errors |


### syslog-server


#### Help

```sh
❯ syslog-server --help
syslog-server 0.1.0
Simple Syslog server for testing & development

Currently TCP and TLS transports are supported, UDP might be added in the future. Received
messages will be logged to stdout. Set the environmental variable variable `SYSLOG_SERVER_LOG`
to one of the values (from quiet to verbose) `error`, `warn`, `info`, `debug` or `trace` to log
more or less information.

USAGE:
    syslog-server [OPTIONS] <transport>

FLAGS:
    -h, --help
            Prints help information

    -V, --version
            Prints version information


OPTIONS:
        --address <address>
            Address to listen on [default: [::]]

        --cert <certificate>
            Path to file containing TLS certificate

        --port <port>
            Port to listen on [default: 514]

        --key <private-key>
            Path to file containing TLS private key


ARGS:
    <transport>
            Syslog Protocol to accept [possible values: tcp, udp, tls]
```


#### Examples

| Command | Description |
| --- | --- |
| `syslog-server tls --cert cert.pem --key.pem --port 5014` | Listen for TLS connections on port 5014 using the given key & certificate |
| `SYSLOG_SERVER_LOG=debug syslog-server tcp --port 5014` | Listen for TCP connection on port 5014, being extra verbose about it |


## Development

Run `nix develop` for a development shell.

| Command | Description |
| --- | --- |
| `nix develop` | Development shell including all dependencies |
| `nix build '.#docker' && docker load <result` | Build and load a docker image |
| `nix build '.#static'` | Build static (musl) binaries |
| `nix build '.#native'` | Build dynamically linked binaries |
| `just gen-selfsigned-cert` | Generate a self-signed certificate for use with the TLS server |
