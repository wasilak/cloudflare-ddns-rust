use axum::response::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct Response {
    ip: String,
    source: String,
}

#[axum::debug_handler]
pub async fn root_handler() -> Json<Response> {
    let (ip, source) = match crate::libs::ip::get_ip().await {
        Ok(result) => result,
        Err(e) => {
            tracing::error!("Failed to get IP: {}", e);
            return Json(Response {
                ip: "".to_string(),
                source: "error".to_string(),
            });
        }
    };

    let response = Response {
        ip,
        source: source.name().to_string(),
    };

    Json(response)
}
