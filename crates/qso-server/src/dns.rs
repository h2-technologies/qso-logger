use qso_core::config::CloudflareConfig;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct CloudflareDns {
    client: Client,
    api_token: String,
    zone_id_forward: String,
    zone_id_reverse: String,
}

#[derive(Serialize)]
struct DnsRecord {
    #[serde(rename = "type")]
    record_type: String,
    name: String,
    content: String,
    ttl: u32,
    proxied: bool,
}

#[derive(Serialize)]
struct SrvData {
    service: String,
    proto: String,
    name: String,
    priority: u16,
    weight: u16,
    port: u16,
    target: String,
}

#[derive(Serialize)]
struct SrvRecord {
    #[serde(rename = "type")]
    record_type: String,
    name: String,
    data: SrvData,
    ttl: u32,
}

#[derive(Deserialize)]
struct CfResponse {
    success: bool,
    errors: Vec<CfError>,
}

#[derive(Deserialize)]
struct CfError {
    message: String,
}

#[derive(Deserialize)]
struct CfListResponse {
    success: bool,
    result: Vec<CfDnsRecord>,
}

#[derive(Deserialize)]
struct CfDnsRecord {
    id: String,
    name: String,
    #[serde(rename = "type")]
    record_type: String,
}

impl CloudflareDns {
    pub fn new(cfg: &CloudflareConfig) -> Self {
        Self {
            client: Client::new(),
            api_token: cfg.api_token.clone(),
            zone_id_forward: cfg.zone_id_forward.clone(),
            zone_id_reverse: cfg.zone_id_reverse.clone(),
        }
    }

    fn base_url(&self, zone_id: &str) -> String {
        format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            zone_id
        )
    }

    async fn find_existing(&self, zone_id: &str, record_type: &str, name: &str) -> Option<String> {
        let url = format!(
            "{}?type={}&name={}",
            self.base_url(zone_id),
            record_type,
            name
        );
        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.api_token)
            .send()
            .await
            .ok()?;
        let list: CfListResponse = resp.json().await.ok()?;
        if list.success {
            list.result
                .into_iter()
                .find(|r| r.name == name && r.record_type == record_type)
                .map(|r| r.id)
        } else {
            None
        }
    }

    pub async fn upsert_aaaa(&self, name: &str, ipv6: &str, ttl: u32) -> Result<(), String> {
        let zone = self.zone_id_forward.clone();
        let existing = self.find_existing(&zone, "AAAA", name).await;
        let record = DnsRecord {
            record_type: "AAAA".to_string(),
            name: name.to_string(),
            content: ipv6.to_string(),
            ttl,
            proxied: false,
        };
        self.upsert_record(&zone, existing.as_deref(), &record)
            .await
    }

    pub async fn upsert_srv(
        &self,
        name: &str,
        priority: u16,
        weight: u16,
        port: u16,
        target: &str,
        ttl: u32,
    ) -> Result<(), String> {
        let zone = self.zone_id_forward.clone();
        let existing = self.find_existing(&zone, "SRV", name).await;
        let parts: Vec<&str> = name.splitn(3, '.').collect();
        // SRV names are expected in the form `_service._proto.hostname`.
        // If the name doesn't conform, return an error rather than silently
        // using incorrect fallback values.
        if parts.len() != 3 {
            return Err(format!(
                "SRV name '{}' is not in expected '_service._proto.host' format",
                name
            ));
        }
        let (service, proto, host_name) = (parts[0], parts[1], parts[2]);
        let record = SrvRecord {
            record_type: "SRV".to_string(),
            name: name.to_string(),
            ttl,
            data: SrvData {
                service: service.to_string(),
                proto: proto.to_string(),
                name: host_name.to_string(),
                priority,
                weight,
                port,
                target: format!("{}.", target),
            },
        };
        if let Some(id) = existing {
            let url = format!("{}/{}", self.base_url(&zone), id);
            let resp = self
                .client
                .put(&url)
                .bearer_auth(&self.api_token)
                .json(&record)
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let cf: CfResponse = resp.json().await.map_err(|e| e.to_string())?;
            if !cf.success {
                return Err(cf
                    .errors
                    .into_iter()
                    .map(|e| e.message)
                    .collect::<Vec<_>>()
                    .join(", "));
            }
        } else {
            let url = self.base_url(&zone);
            let resp = self
                .client
                .post(&url)
                .bearer_auth(&self.api_token)
                .json(&record)
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let cf: CfResponse = resp.json().await.map_err(|e| e.to_string())?;
            if !cf.success {
                return Err(cf
                    .errors
                    .into_iter()
                    .map(|e| e.message)
                    .collect::<Vec<_>>()
                    .join(", "));
            }
        }
        Ok(())
    }

    pub async fn upsert_ptr(&self, name: &str, target: &str, ttl: u32) -> Result<(), String> {
        let zone = self.zone_id_reverse.clone();
        let existing = self.find_existing(&zone, "PTR", name).await;
        let record = DnsRecord {
            record_type: "PTR".to_string(),
            name: name.to_string(),
            content: format!("{}.", target),
            ttl,
            proxied: false,
        };
        self.upsert_record(&zone, existing.as_deref(), &record)
            .await
    }

    async fn upsert_record<T: serde::Serialize>(
        &self,
        zone: &str,
        existing_id: Option<&str>,
        record: &T,
    ) -> Result<(), String> {
        let resp = if let Some(id) = existing_id {
            let url = format!("{}/{}", self.base_url(zone), id);
            self.client
                .put(&url)
                .bearer_auth(&self.api_token)
                .json(record)
                .send()
                .await
        } else {
            let url = self.base_url(zone);
            self.client
                .post(&url)
                .bearer_auth(&self.api_token)
                .json(record)
                .send()
                .await
        }
        .map_err(|e| e.to_string())?;

        let cf: CfResponse = resp.json().await.map_err(|e| e.to_string())?;
        if !cf.success {
            return Err(cf
                .errors
                .into_iter()
                .map(|e| e.message)
                .collect::<Vec<_>>()
                .join(", "));
        }
        Ok(())
    }
}
