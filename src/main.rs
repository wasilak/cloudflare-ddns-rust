use axum::{Router, response::Json, routing::get};
use serde::Serialize;
use tracing::info;
use tracing_subscriber::FmtSubscriber;

#[derive(Serialize)]
struct Response {
    message: String,
}

#[tokio::main]
async fn main() {
    // structured JSON logging
    let subscriber = FmtSubscriber::builder()
        .pretty()
        .with_level(true)
        .with_line_number(true)
        .json()
        .with_env_filter("info")
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let app = Router::new();

    let app = app.route("/", get(handler));

    let listener = match tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("failed to bind")
    {
        listener => listener,
    };

    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler() -> Json<Response> {
    info!("Handling request");
    let response = Response {
        message: "Hello, World!".to_string(),
    };
    Json(response)
}
