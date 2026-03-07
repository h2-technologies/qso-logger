use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub callsign: String,
    pub ipv6_address: String,
    pub subnet: u16,
    pub tcp_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub success: bool,
    pub message: String,
    pub aaaa_record: Option<String>,
    pub srv_record: Option<String>,
    pub ptr_record: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub version: String,
    pub prefix: String,
    pub multicast_global: String,
    pub multicast_site_local: String,
    pub multicast_link_local: String,
}
