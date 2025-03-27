use cloudflare::framework::client::ClientConfig;
use cloudflare::framework::client::async_api::Client as AsyncClient;
use cloudflare::framework::{Environment, auth::Credentials};
use once_cell::sync::{Lazy, OnceCell};
use std::collections::HashMap;
use std::sync::RwLock;

pub static API_CLIENT: OnceCell<AsyncClient> = OnceCell::new();

pub fn get_api_client() -> &'static AsyncClient {
    API_CLIENT.get().unwrap()
}

static ZONE_ID_CACHE: Lazy<RwLock<HashMap<String, String>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct DnsRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxied: Option<bool>,
}

pub async fn init_cf(
    cf_email: String,
    cf_api_key: String,
) -> Result<(), cloudflare::framework::Error> {
    let environment = Environment::Production;

    let credentials: Credentials = Credentials::UserAuthKey {
        email: cf_email,
        key: cf_api_key,
    };

    let client = AsyncClient::new(credentials, ClientConfig::default(), environment)?;

    if API_CLIENT.set(client).is_err() {
        tracing::warn!("API client was already initialized");
    }

    Ok(())
}

pub async fn get_zone(zone_name: String) -> Result<String, Box<dyn std::error::Error>> {
    // First: check if zone_id is cached
    if let Some(cached) = ZONE_ID_CACHE.read().unwrap().get(&zone_name) {
        return Ok(cached.clone());
    }

    // If not cached, fetch from Cloudflare
    match crate::libs::cf::get_zone(&get_api_client(), zone_name.clone()).await {
        Ok(zone_id) => {
            // Save to cache
            ZONE_ID_CACHE
                .write()
                .unwrap()
                .insert(zone_name.clone(), zone_id.clone());
            Ok(zone_id)
        }
        Err(e) => {
            tracing::error!("Failed to get zone: {}", e);
            Err(e)
        }
    }
}

pub async fn list_records(zone_id: String) -> Result<Vec<DnsRecord>, Box<dyn std::error::Error>> {
    match crate::libs::cf::list_records(&get_api_client(), zone_id).await {
        Ok(records) => return Ok(records),
        Err(e) => {
            tracing::error!("Failed to list records: {}", e);
            return Err(e);
        }
    }
}

pub async fn get_record(
    zone_id: String,
    record: String,
) -> Result<DnsRecord, Box<dyn std::error::Error>> {
    match crate::libs::cf::list_records(&get_api_client(), zone_id).await {
        Ok(records) => {
            return {
                let record = records.iter().find(|r| r.name == Some(record.clone()));
                match record {
                    Some(record) => Ok(record.clone()),
                    None => Err("Record not found".into()),
                }
            };
        }
        Err(e) => {
            tracing::error!("Failed to list records: {}", e);
            return Err(e);
        }
    }
}

pub async fn create_record(
    zone_id: String,
    record: DnsRecord,
) -> Result<DnsRecord, Box<dyn std::error::Error>> {
    match crate::libs::cf::create_record(&get_api_client(), zone_id, record).await {
        Ok(record) => return Ok(record),
        Err(e) => {
            tracing::error!("Failed to create record: {}", e);
            return Err(e);
        }
    }
}

pub async fn update_record(
    zone_id: String,
    record: DnsRecord,
) -> Result<DnsRecord, Box<dyn std::error::Error>> {
    match crate::libs::cf::update_record(&get_api_client(), zone_id, record).await {
        Ok(record) => return Ok(record),
        Err(e) => {
            tracing::error!("Failed to update record: {}", e);
            return Err(e);
        }
    }
}

pub async fn delete_record(
    zone_id: String,
    record: DnsRecord,
) -> Result<DnsRecord, Box<dyn std::error::Error>> {
    match crate::libs::cf::delete_record(&get_api_client(), zone_id, record).await {
        Ok(record) => return Ok(record),
        Err(e) => {
            tracing::error!("Failed to delete record: {}", e);
            return Err(e);
        }
    }
}
