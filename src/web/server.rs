use super::routes::root_handler;
use axum::{Router, routing::get};
use tower_http::trace::{DefaultOnRequest, TraceLayer};
// use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse};
use tracing::Level;

pub struct Server {}

impl Server {
    pub async fn init() {
        let app = Router::new().layer(TraceLayer::new_for_http());

        let app = app.route("/", get(root_handler)).layer(
            TraceLayer::new_for_http()
                // .make_span_with(DefaultMakeSpan::new().level(Level::INFO)) // to verbose, for now
                .on_request(DefaultOnRequest::new().level(Level::INFO)), // .on_response(DefaultOnResponse::new().level(Level::INFO)),
        );

        let listener = match tokio::net::TcpListener::bind("127.0.0.1:3000")
            .await
            .expect("failed to bind")
        {
            listener => listener,
        };

        println!("listening on {}", listener.local_addr().unwrap());
        axum::serve(listener, app).await.unwrap();
    }
}
