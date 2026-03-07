use is_terminal::IsTerminal;
use std::io;

fn main() {
    // When stdin is a terminal the program is being run interactively (e.g. as a
    // foreground service from a shell), so start the server.  When stdin is not
    // attached to a terminal (e.g. launched by a desktop environment without a
    // controlling TTY), launch the GUI client instead.
    if io::stdin().is_terminal() {
        run_server();
    } else {
        run_client();
    }
}

fn run_server() {
    eprintln!("[qso-logger] Starting in server mode...");
    eprintln!(
        "[qso-logger] Run 'qso-server' binary for the full server, or use: cargo run -p qso-server"
    );
    eprintln!("[qso-logger] Config file: config.toml (copy config.example.toml to get started)");

    let config = qso_core::config::Config::load_or_default("config.toml");
    eprintln!(
        "[qso-logger] Server will bind to {}:{}",
        config.server.bind_address, config.server.port
    );

    let prefix = format!(
        "{:04x}:{:04x}:{:04x}::/48",
        qso_core::ipv6::IPV6_PREFIX[0],
        qso_core::ipv6::IPV6_PREFIX[1],
        qso_core::ipv6::IPV6_PREFIX[2],
    );
    eprintln!("[qso-logger] IPv6 prefix: {}", prefix);
    eprintln!(
        "[qso-logger] Global multicast: {}",
        qso_core::ipv6::multicast_global(qso_core::ipv6::MULTICAST_GROUP_ALL_STATIONS)
    );
    eprintln!(
        "[qso-logger] Site-local multicast: {}",
        qso_core::ipv6::multicast_site_local(qso_core::ipv6::MULTICAST_GROUP_ALL_STATIONS)
    );
    eprintln!(
        "[qso-logger] Link-local multicast: {}",
        qso_core::ipv6::multicast_link_local(qso_core::ipv6::MULTICAST_GROUP_ALL_STATIONS)
    );

    eprintln!("[qso-logger] To start the full server, run: cargo run -p qso-server");
}

fn run_client() {
    eprintln!("[qso-logger] GUI mode: Launch 'qso-client' binary for the Tauri GUI");
    eprintln!("[qso-logger] For development: cd crates/qso-client && cargo tauri dev");
}
