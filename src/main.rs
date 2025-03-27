use clap::Parser;

mod libs;
mod web;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Cloudflare API key
    #[arg(env = "CF_API_KEY")]
    cf_api_key: String,

    /// Cloudflare API email
    #[arg(env = "CF_API_EMAIL")]
    cf_api_email: String,

    /// webserver enabled
    #[arg(short, long, default_value_t = false)]
    web_server: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    libs::logging::Logger::init();

    match libs::api::init_cf(args.cf_api_email, args.cf_api_key).await {
        Ok(_) => {
            tracing::info!("Cloudflare API client initialized");
        }
        Err(e) => {
            tracing::error!("Failed to initialize Cloudflare API client: {}", e);
        }
    }

    match crate::libs::ip::IPSource::get().await {
        Ok(ip) => {
            tracing::info!("IP: {} from {}", ip.ip, ip.source.name());
            crate::libs::ip::set_external_ip(ip);
        }
        Err(e) => tracing::error!("Failed to get IP: {}", e),
    }

    if args.web_server {
        web::server::Server::init().await;
    }

    Ok(())
}
