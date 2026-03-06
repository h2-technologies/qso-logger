use std::{
    net::{Ipv6Addr, SocketAddr},
    sync::Arc,
};

use axum::{
    Router,
    extract::{
        State,
        ws::{Message as AxumMessage, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
    routing::get,
};
use axum_server::tls_rustls::RustlsConfig;
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use native_tls::TlsConnector;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use tokio_tungstenite::{
    Connector, connect_async, connect_async_tls_with_config,
    tungstenite::Message as TungsteniteMessage,
};

const AMATEUR_PREFIX_SEGMENTS: [u16; 3] = [0x2602, 0xfa86, 0x0044];
const PREFIX_LENGTH_BITS: u8 = 48;
const TUNNEL_BUFFER_SIZE: usize = 16 * 1024;

type AppResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, Clone, Parser)]
#[command(author, version, about = "QSO Logger IPv6 HTTPS/WSS tunnel")]
struct Cli {
    #[arg(long, conflicts_with = "client", help = "Run in tunnel server mode")]
    server: bool,

    #[arg(long, conflicts_with = "server", help = "Run in tunnel client mode")]
    client: bool,

    #[arg(
        long,
        default_value = "[::]:8443",
        help = "Server bind address when --server is used"
    )]
    bind: String,

    #[arg(
        long,
        default_value = "[::1]:9000",
        help = "Upstream TCP endpoint for server-side tunnel forwarding"
    )]
    upstream: String,

    #[arg(
        long,
        default_value = "cert.pem",
        help = "Server TLS certificate PEM file"
    )]
    cert: String,

    #[arg(
        long,
        default_value = "key.pem",
        help = "Server TLS private key PEM file"
    )]
    key: String,

    #[arg(
        long,
        default_value = "wss://[::1]:8443/tunnel",
        help = "Tunnel websocket URL used by client mode"
    )]
    server_url: String,

    #[arg(
        long,
        default_value = "[::1]:7000",
        help = "Local TCP listener for client mode"
    )]
    listen: String,

    #[arg(
        long,
        default_value_t = false,
        help = "Skip TLS certificate validation in client mode"
    )]
    insecure: bool,
}

#[derive(Clone)]
struct ServerState {
    upstream: String,
}

#[tokio::main]
async fn main() -> AppResult<()> {
    let cli = Cli::parse();

    if cli.server == cli.client {
        return Err(std::io::Error::other(
            "either --server or --client must be specified, but not both",
        )
        .into());
    }

    let sample_unicast = amateur_global_unicast(1, 1);
    let site_local_group = site_local_multicast_group(1);
    let global_group = global_multicast_group(1);
    println!(
        "Amateur IPv6 /{} prefix 2602:fa86:44:: example={} site-local-mcast={} global-mcast={}",
        PREFIX_LENGTH_BITS, sample_unicast, site_local_group, global_group
    );

    if cli.server {
        run_server(cli).await
    } else {
        run_client(cli).await
    }
}

fn amateur_global_unicast(subnet_id: u16, interface_id: u64) -> Ipv6Addr {
    Ipv6Addr::new(
        AMATEUR_PREFIX_SEGMENTS[0],
        AMATEUR_PREFIX_SEGMENTS[1],
        AMATEUR_PREFIX_SEGMENTS[2],
        subnet_id,
        (interface_id >> 48) as u16,
        (interface_id >> 32) as u16,
        (interface_id >> 16) as u16,
        interface_id as u16,
    )
}

fn unicast_prefix_multicast(scope: u8, group_id: u32) -> Ipv6Addr {
    Ipv6Addr::new(
        0xff30 | u16::from(scope & 0x0f),
        PREFIX_LENGTH_BITS.into(),
        AMATEUR_PREFIX_SEGMENTS[0],
        AMATEUR_PREFIX_SEGMENTS[1],
        AMATEUR_PREFIX_SEGMENTS[2],
        0x0000,
        (group_id >> 16) as u16,
        group_id as u16,
    )
}

fn site_local_multicast_group(group_id: u32) -> Ipv6Addr {
    unicast_prefix_multicast(0x5, group_id)
}

fn global_multicast_group(group_id: u32) -> Ipv6Addr {
    unicast_prefix_multicast(0xE, group_id)
}

async fn run_server(cli: Cli) -> AppResult<()> {
    let state = Arc::new(ServerState {
        upstream: cli.upstream.clone(),
    });

    let app = Router::new()
        .route(
            "/",
            get(|| async { "qso-logger tunnel server over HTTPS/WSS" }),
        )
        .route("/tunnel", get(ws_upgrade))
        .with_state(state);

    let tls = RustlsConfig::from_pem_file(&cli.cert, &cli.key).await?;
    println!("server listening on https://{}", cli.bind);
    let bind_addr: SocketAddr = cli.bind.parse()?;
    axum_server::bind_rustls(bind_addr, tls)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn ws_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ServerState>>,
) -> impl IntoResponse {
    let upstream = state.upstream.clone();
    ws.on_upgrade(move |socket| tunnel_websocket_to_tcp(socket, upstream))
}

async fn tunnel_websocket_to_tcp(websocket: WebSocket, upstream: String) {
    if let Err(err) = tunnel_websocket_to_tcp_inner(websocket, &upstream).await {
        eprintln!("websocket tunnel error: {err}");
    }
}

async fn tunnel_websocket_to_tcp_inner(websocket: WebSocket, upstream: &str) -> AppResult<()> {
    let stream = TcpStream::connect(upstream).await?;
    pipe_websocket_and_tcp(websocket, stream).await
}

async fn run_client(cli: Cli) -> AppResult<()> {
    let listener = TcpListener::bind(&cli.listen).await?;
    println!(
        "client listening on tcp://{} and tunneling to {}",
        cli.listen, cli.server_url
    );

    loop {
        let (stream, addr) = listener.accept().await?;
        let server_url = cli.server_url.clone();
        let insecure = cli.insecure;
        tokio::spawn(async move {
            if let Err(err) = forward_local_stream(stream, &server_url, insecure).await {
                eprintln!("client tunnel error from {addr}: {err}");
            }
        });
    }
}

async fn forward_local_stream(
    stream: TcpStream,
    server_url: &str,
    insecure: bool,
) -> AppResult<()> {
    let (websocket, _) = if insecure {
        let mut builder = TlsConnector::builder();
        builder.danger_accept_invalid_certs(true);
        builder.danger_accept_invalid_hostnames(true);
        let connector = Connector::NativeTls(builder.build()?);
        connect_async_tls_with_config(server_url, None, false, Some(connector)).await?
    } else {
        connect_async(server_url).await?
    };

    pipe_tungstenite_and_tcp(websocket, stream).await
}

async fn pipe_websocket_and_tcp(websocket: WebSocket, stream: TcpStream) -> AppResult<()> {
    let (mut ws_sink, mut ws_stream) = websocket.split();
    let (mut tcp_read, mut tcp_write) = stream.into_split();

    let ws_to_tcp = async {
        while let Some(message) = ws_stream.next().await {
            match message? {
                AxumMessage::Binary(data) => tcp_write.write_all(&data).await?,
                AxumMessage::Text(data) => tcp_write.write_all(data.as_bytes()).await?,
                AxumMessage::Close(_) => break,
                AxumMessage::Ping(_) | AxumMessage::Pong(_) => {}
            }
        }
        tcp_write.shutdown().await?;
        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    };

    let tcp_to_ws = async {
        let mut buf = [0_u8; TUNNEL_BUFFER_SIZE];
        loop {
            let read = tcp_read.read(&mut buf).await?;
            if read == 0 {
                break;
            }
            ws_sink
                .send(AxumMessage::Binary(buf[..read].to_vec().into()))
                .await?;
        }
        ws_sink.send(AxumMessage::Close(None)).await?;
        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    };

    tokio::try_join!(ws_to_tcp, tcp_to_ws)?;
    Ok(())
}

async fn pipe_tungstenite_and_tcp(
    websocket: tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>,
    stream: TcpStream,
) -> AppResult<()> {
    let (mut ws_sink, mut ws_stream) = websocket.split();
    let (mut tcp_read, mut tcp_write) = stream.into_split();

    let ws_to_tcp = async {
        while let Some(message) = ws_stream.next().await {
            match message? {
                TungsteniteMessage::Binary(data) => tcp_write.write_all(&data).await?,
                TungsteniteMessage::Text(data) => tcp_write.write_all(data.as_bytes()).await?,
                TungsteniteMessage::Close(_) => break,
                TungsteniteMessage::Ping(_) | TungsteniteMessage::Pong(_) => {}
                TungsteniteMessage::Frame(_) => {}
            }
        }
        tcp_write.shutdown().await?;
        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    };

    let tcp_to_ws = async {
        let mut buf = [0_u8; TUNNEL_BUFFER_SIZE];
        loop {
            let read = tcp_read.read(&mut buf).await?;
            if read == 0 {
                break;
            }
            ws_sink
                .send(TungsteniteMessage::Binary(buf[..read].to_vec().into()))
                .await?;
        }
        ws_sink.send(TungsteniteMessage::Close(None)).await?;
        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    };

    tokio::try_join!(ws_to_tcp, tcp_to_ws)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn derives_amateur_unicast_from_required_prefix() {
        let addr = amateur_global_unicast(0x0123, 0x4455_6677_8899_aabb);
        assert_eq!(addr.to_string(), "2602:fa86:44:123:4455:6677:8899:aabb");
    }

    #[test]
    fn derives_site_local_and_global_multicast_groups() {
        let site_local = site_local_multicast_group(0x0000_0042);
        let global = global_multicast_group(0x0000_0042);

        assert_eq!(site_local.to_string(), "ff35:30:2602:fa86:44::42");
        assert_eq!(global.to_string(), "ff3e:30:2602:fa86:44::42");
    }

    #[test]
    fn cli_supports_server_and_client_flags() {
        let cmd = Cli::command();
        cmd.debug_assert();

        assert!(Cli::try_parse_from(["qso-logger", "--server"]).is_ok());
        assert!(Cli::try_parse_from(["qso-logger", "--client"]).is_ok());
        assert!(Cli::try_parse_from(["qso-logger", "--server", "--client"]).is_err());
    }
}
