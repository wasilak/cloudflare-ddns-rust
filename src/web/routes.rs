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
    let record = match crate::libs::api::get_record(&zone_name, &record).await {
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

    // Prepare payload
    payload.name = Some(record.clone());
    payload.content = Some(ip.ip.clone());

    // Perform upsert using your unified logic
    let result = crate::libs::api::upsert_record(&zone_name, payload).await;

    match result {
        Ok(updated) => Json(Response {
            ip: Some(ip),
            records: Some(vec![updated]),
            error: None,
        }),
        Err(e) => Json(Response {
            ip: Some(ip),
            records: None,
            error: Some(format!("Upsert failed: {}", e)),
        }),
    }
}

#[axum::debug_handler]
pub async fn list_handler(Path(zone_name): Path<String>) -> Json<Response> {
    let records = match crate::libs::api::list_records(&zone_name).await {
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
    let ip = crate::libs::ip::get_external_ip().unwrap();

    let record = match crate::libs::api::delete_record(&zone_name, &record).await {
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
