use qso_core::{
    config::Config,
    ipv6,
    types::{RegisterRequest, RegisterResponse, StatusResponse},
};
use rocket::{get, post, routes, serde::json::Json, State};

#[get("/status")]
pub async fn status() -> Json<StatusResponse> {
    Json(StatusResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
        prefix: format!(
            "{:04x}:{:04x}:{:04x}:{:04x}::/64",
            ipv6::IPV6_PREFIX[0],
            ipv6::IPV6_PREFIX[1],
            ipv6::IPV6_PREFIX[2],
            ipv6::GLOBAL_UNICAST_SUBNET
        ),
        multicast_global: ipv6::multicast_global(ipv6::MULTICAST_GROUP_ALL_STATIONS).to_string(),
        multicast_site_local: ipv6::multicast_site_local(ipv6::MULTICAST_GROUP_ALL_STATIONS)
            .to_string(),
        multicast_link_local: ipv6::multicast_link_local(ipv6::MULTICAST_GROUP_ALL_STATIONS)
            .to_string(),
    })
}

#[post("/register", data = "<req>")]
pub async fn register(
    req: Json<RegisterRequest>,
    config: &State<Config>,
) -> Json<RegisterResponse> {
    let callsign = req.callsign.to_uppercase();

    let ipv6_addr = match req.ipv6_address.parse::<std::net::Ipv6Addr>() {
        Ok(addr) => addr,
        Err(_) => {
            return Json(RegisterResponse {
                success: false,
                message: "Invalid IPv6 address".to_string(),
                aaaa_record: None,
                srv_record: None,
                ptr_record: None,
            });
        }
    };

    let aaaa_name = format!("{}.{}", callsign.to_lowercase(), config.server.domain);
    let srv_name = format!(
        "_qso._tcp.{}.{}",
        callsign.to_lowercase(),
        config.server.domain
    );
    let ptr_name = ipv6::reverse_dns_record_name(&ipv6_addr);

    let dns_client = crate::dns::CloudflareDns::new(&config.cloudflare);
    let mut dns_errors = Vec::new();

    match dns_client
        .upsert_aaaa(&aaaa_name, &req.ipv6_address, config.cloudflare.ttl)
        .await
    {
        Ok(_) => {}
        Err(e) => dns_errors.push(format!("AAAA: {}", e)),
    }

    match dns_client
        .upsert_srv(
            &srv_name,
            10,
            10,
            req.tcp_port,
            &aaaa_name,
            config.cloudflare.ttl,
        )
        .await
    {
        Ok(_) => {}
        Err(e) => dns_errors.push(format!("SRV: {}", e)),
    }

    match dns_client
        .upsert_ptr(&ptr_name, &aaaa_name, config.cloudflare.ttl)
        .await
    {
        Ok(_) => {}
        Err(e) => dns_errors.push(format!("PTR: {}", e)),
    }

    let message = if dns_errors.is_empty() {
        "Registration successful".to_string()
    } else {
        format!("Partial success. DNS errors: {}", dns_errors.join("; "))
    };

    Json(RegisterResponse {
        success: dns_errors.is_empty(),
        message,
        aaaa_record: Some(aaaa_name),
        srv_record: Some(srv_name),
        ptr_record: Some(ptr_name),
    })
}

pub fn routes() -> Vec<rocket::Route> {
    routes![status, register]
}
