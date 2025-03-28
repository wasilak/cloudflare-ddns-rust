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

    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_type: Option<String>,
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

pub async fn get_zone(zone_name: &String) -> Result<String, Box<dyn std::error::Error>> {
    // First: check if zone_id is cached
    if let Some(cached) = ZONE_ID_CACHE.read().unwrap().get(zone_name) {
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

pub async fn list_records(
    zone_name: &String,
) -> Result<Vec<DnsRecord>, Box<dyn std::error::Error>> {
    let config = crate::libs::config::CONFIG.read().unwrap();

    match config.get_zone_records(&zone_name) {
        Some(records) => Ok(records.clone()),
        None => Err(format!("No records found for zone {}", zone_name).into()),
    }
}

pub async fn get_record(
    zone_name: &String,
    record: &String,
) -> Result<DnsRecord, Box<dyn std::error::Error>> {
    let config = crate::libs::config::CONFIG.read().unwrap();

    match config.get_zone_record(&zone_name, &record) {
        Some(records) => Ok(records.clone()),
        None => Err(format!("Record {} not found in zone {}", record, zone_name).into()),
    }
}

pub async fn upsert_record(
    zone_name: &String,
    record: DnsRecord,
) -> Result<DnsRecord, Box<dyn std::error::Error>> {
    let zone_id = match crate::libs::api::get_zone(zone_name).await {
        Ok(zone_id) => zone_id,
        Err(e) => {
            tracing::error!("Failed to get zone: {}", e);
            return Err(e);
        }
    };

    let exists = {
        let config = crate::libs::config::CONFIG.read().unwrap();
        config
            .get_zone_record(zone_name, &record.clone().name.unwrap())
            .is_some()
    };

    let result = if exists {
        crate::libs::cf::update_record(&get_api_client(), zone_id, record).await
    } else {
        crate::libs::cf::create_record(&get_api_client(), zone_id, record).await
    };

    let record = match result {
        Ok(record) => record,
        Err(e) => {
            tracing::error!(
                "Failed to {} record: {}",
                if exists { "update" } else { "create" },
                e
            );
            return Err(e);
        }
    };

    let mut config = crate::libs::config::CONFIG.write().unwrap();
    match config.upsert_zone_record(zone_name, record.clone()) {
        Ok(_) => Ok(record),
        Err(e) => Err(e),
    }
}

pub async fn delete_record(
    zone_name: &String,
    record: &String,
) -> Result<DnsRecord, Box<dyn std::error::Error>> {
    let zone_id = match crate::libs::api::get_zone(zone_name).await {
        Ok(zone_id) => zone_id.clone(),
        Err(e) => {
            tracing::error!("Failed to get zone: {}", e);
            return Err(e);
        }
    };

    let record = match get_record(&zone_name, record).await {
        Ok(record) => record,
        Err(e) => {
            tracing::error!("Failed to get record: {}", e);
            return Err(e);
        }
    };

    let record = match crate::libs::cf::delete_record(&get_api_client(), zone_id, record).await {
        Ok(record) => record,
        Err(e) => {
            tracing::error!("Failed to delete record: {}", e);
            return Err(e);
        }
    };

    let mut config = crate::libs::config::CONFIG.write().unwrap();
    match config.delete_zone_record(zone_name, &record.clone().name.unwrap()) {
        Ok(_) => Ok(record),
        Err(e) => Err(e),
    }
}
