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

    let existing_record = match crate::libs::api::get_record(zone_id.unwrap(), record.clone()).await
    {
        Ok(record) => Some(record),
        Err(_) => None,
    };

    let ip = crate::libs::ip::get_external_ip().unwrap();

    let mut records = vec![];

    if !existing_record.is_none() {
        let existing_record = existing_record.unwrap();
        records.push(existing_record.clone());
        payload.name = existing_record.name.clone();
    } else {
        payload.name = Some(record.clone());
    }

    payload.content = Some(ip.ip.to_string());

    tracing::info!("Upserting record: {:#?}", payload);

    records.push(payload);

    let response = Response {
        ip: Some(ip),
        records: Some(records),
        error: None,
    };

    Json(response)
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

    let records = match crate::libs::api::list_records(zone_id.unwrap()).await {
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
