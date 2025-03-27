use axum::{Json, extract::Path};

#[derive(serde::Serialize)]
pub struct ResponseRoot {
    message: String,
}

#[derive(serde::Serialize)]
pub struct Response {
    #[serde(skip_serializing_if = "Option::is_none")]
    ip: Option<crate::libs::ip::IP>,

    #[serde(skip_serializing_if = "Option::is_none")]
    records: Option<Vec<crate::libs::api::DnsRecord>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[axum::debug_handler]
pub async fn root_handler() -> Json<ResponseRoot> {
    let response = ResponseRoot {
        message: "Hello, World!".to_string(),
    };

    Json(response)
}

#[axum::debug_handler]
pub async fn get_record_handler(
    Path((zone_name, record)): Path<(String, String)>,
) -> Json<Response> {
    let zone_id = match crate::libs::api::get_zone(zone_name).await {
        Ok(zone_id) => Some(zone_id),
        Err(e) => {
            tracing::error!("Failed to get zone: {}", e);
            return Json(Response {
                ip: None,
                records: None,
                error: Some(e.to_string()),
            });
        }
    };

    let record = match crate::libs::api::get_record(zone_id.unwrap(), record).await {
        Ok(record) => record,
        Err(e) => {
            tracing::error!("Failed to get record: {}", e);
            return Json(Response {
                ip: None,
                records: None,
                error: Some(e.to_string()),
            });
        }
    };

    let ip = crate::libs::ip::get_external_ip().unwrap();

    let response = Response {
        ip: Some(ip.clone()),
        records: Some(Vec::from([record])),
        error: None,
    };

    Json(response)
}

#[axum::debug_handler]
pub async fn upsert_record_handler(
    Path((zone_name, record)): Path<(String, String)>,
    Json(mut payload): Json<crate::libs::api::DnsRecord>,
) -> Json<Response> {
    tracing::info!("POST payload: {:#?}", payload);

    // Get the zone ID
    let zone_id = match crate::libs::api::get_zone(zone_name.clone()).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("Failed to get zone: {}", e);
            return Json(Response {
                ip: None,
                records: None,
                error: Some(format!("Zone not found: {}", e)),
            });
        }
    };

    // Check if the record exists
    let maybe_record = crate::libs::api::get_record(zone_id.clone(), record.clone())
        .await
        .ok();

    // Get current external IP
    let ip = match crate::libs::ip::get_external_ip() {
        Some(ip) => ip,
        None => {
            return Json(Response {
                ip: None,
                records: None,
                error: Some("Could not retrieve external IP".into()),
            });
        }
    };

    let mut records = Vec::new();

    payload.name = Some(record.clone());
    payload.content = Some(ip.ip.clone());

    if let Some(mut existing) = maybe_record {
        tracing::info!("Record exists, updating");

        // You could diff/merge here if needed
        existing.content = payload.content.clone();
        existing.ttl = payload.ttl;
        existing.proxied = payload.proxied;

        let record = crate::libs::api::update_record(zone_id, existing.clone())
            .await
            .unwrap();
        records.push(record);
    } else {
        tracing::info!("Record does not exist, creating");

        let record = crate::libs::api::create_record(zone_id, payload.clone())
            .await
            .unwrap();
        records.push(record);
    }

    Json(Response {
        ip: Some(ip),
        records: Some(records),
        error: None,
    })
}

#[axum::debug_handler]
pub async fn list_handler(Path(zone_name): Path<String>) -> Json<Response> {
    let zone_id = match crate::libs::api::get_zone(zone_name).await {
        Ok(zone_id) => Some(zone_id),
        Err(e) => {
            tracing::error!("Failed to get zone: {}", e);
            return Json(Response {
                ip: None,
                records: None,
                error: Some(e.to_string()),
            });
        }
    };

    let records = match crate::libs::api::list_records(zone_id.clone().unwrap()).await {
        Ok(records) => records,
        Err(e) => {
            tracing::error!("Failed to get records: {}", e);
            return Json(Response {
                ip: None,
                records: None,
                error: Some(e.to_string()),
            });
        }
    };

    let ip = crate::libs::ip::get_external_ip().unwrap();

    let response = Response {
        ip: Some(ip),
        records: Some(records),
        error: None,
    };

    Json(response)
}

#[axum::debug_handler]
pub async fn delete_record_handler(
    Path((zone_name, record)): Path<(String, String)>,
) -> Json<Response> {
    let zone_id = match crate::libs::api::get_zone(zone_name).await {
        Ok(zone_id) => Some(zone_id),
        Err(e) => {
            tracing::error!("Failed to get zone: {}", e);
            return Json(Response {
                ip: None,
                records: None,
                error: Some(e.to_string()),
            });
        }
    };

    let record = match crate::libs::api::get_record(zone_id.clone().unwrap(), record).await {
        Ok(record) => record,
        Err(e) => {
            tracing::error!("Failed to get record: {}", e);
            return Json(Response {
                ip: None,
                records: None,
                error: Some(e.to_string()),
            });
        }
    };

    let ip = crate::libs::ip::get_external_ip().unwrap();

    let record = match crate::libs::api::delete_record(zone_id.unwrap(), record).await {
        Ok(record) => record,
        Err(e) => {
            tracing::error!("Failed to delete record: {}", e);
            return Json(Response {
                ip: Some(ip),
                records: None,
                error: Some(e.to_string()),
            });
        }
    };

    let response = Response {
        ip: Some(ip),
        records: Some(Vec::from([record])),
        error: None,
    };

    Json(response)
}
