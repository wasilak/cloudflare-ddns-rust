use clap::{Parser, Subcommand};

mod commands;
mod libs;
mod web;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(name = "cf-dns-manager", version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,

    #[clap(flatten)]
    options: Args,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run with file-based configuration
    File {
        #[arg(short, long, default_value = "./config.yaml")]
        config: String,

        #[arg(short, long, default_value = "127.0.0.1:3000")]
        bind: String,

        #[arg(short, long, default_value = "60")]
        refresh_interval: u64,
    },

    /// Run in API mode (Kubernetes operator or controller)
    Api {
        #[arg(short, long, default_value = "127.0.0.1:3000")]
        bind: String,

        #[arg(short, long, default_value = "60")]
        refresh_interval: u64,
    },
}

#[derive(Debug, Parser)]
struct Args {
    /// Cloudflare API key
    #[arg(env = "CF_API_KEY")]
    cf_api_key: String,

    /// Cloudflare API email
    #[arg(env = "CF_API_EMAIL")]
    cf_api_email: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    libs::logging::Logger::init();

    match libs::api::init_cf(cli.options.cf_api_email, cli.options.cf_api_key).await {
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

    match cli.command {
        Commands::File {
            config,
            bind,
            refresh_interval,
        } => {
            commands::file::run(&config, &bind, refresh_interval).await;
        }
        Commands::Api {
            bind,
            refresh_interval,
        } => {
            commands::api::run(&bind, refresh_interval).await;
        }
    }

    Ok(())
}
