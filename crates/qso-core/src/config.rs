use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub client: ClientConfig,
    pub cloudflare: CloudflareConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub bind_address: String,
    pub port: u16,
    pub qso_tcp_port: u16,
    pub domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub callsign: String,
    pub server_url: String,
    pub subnet: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareConfig {
    pub api_token: String,
    pub zone_id_forward: String,
    pub zone_id_reverse: String,
    pub ttl: u32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server: ServerConfig {
                bind_address: "0.0.0.0".to_string(),
                port: 8080,
                qso_tcp_port: 7300,
                domain: "qso.as40046.org".to_string(),
            },
            client: ClientConfig {
                callsign: "W1AW".to_string(),
                server_url: "http://localhost:8080".to_string(),
                subnet: 0,
            },
            cloudflare: CloudflareConfig {
                api_token: "YOUR_CF_API_TOKEN_HERE".to_string(),
                zone_id_forward: "YOUR_FORWARD_ZONE_ID".to_string(),
                zone_id_reverse: "YOUR_REVERSE_ZONE_ID".to_string(),
                ttl: 3600,
            },
        }
    }
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn load_or_default(path: &str) -> Self {
        Self::load(path).unwrap_or_default()
    }
}
