#![feature(duration_constructors)]

use std::{net::SocketAddr, sync::Arc};

use plugin_hub::abi::plugin_hub::plugin_hub_server::PluginHubServer;
use server::{MyPluginHub, MyPluginHubConfig};

use axum::Router;
use clap::Parser;
use static_files::wrap_files_router;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use figment::{
    providers::{Format, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};

extern crate plugin_hub;

mod axum_handlers;
mod file;
mod server;
mod static_files;

#[derive(Parser, Debug, Serialize, Deserialize)]
#[command(author, version, about)]
struct Config {
    #[arg(short, long, value_parser)]
    addr: SocketAddr,
    #[command(flatten)]
    path_config: MyPluginHubConfig,
}

impl Default for Config {
    fn default() -> Self {
        Figment::new()
            .join(Toml::file("server.toml"))
            .join(Toml::file(
                shellexpand::tilde("~/.config/plugin_hub/server.toml").as_ref(),
            ))
            .join(Toml::file("/etc/plugin_hub/server.toml"))
            .extract()
            .unwrap_or(Config {
                addr: "[::]:3000".parse().unwrap(),
                path_config: MyPluginHubConfig::default(),
            })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Config::try_parse().unwrap_or_default();

    let listener = tokio::net::TcpListener::bind(cli.addr).await.unwrap();
    let greeter = MyPluginHub::new(cli.path_config);

    let arc_greeter = Arc::new(greeter);

    let axum_routers = axum_handlers::router(arc_greeter.clone());
    let svc = tonic::service::Routes::new(PluginHubServer::from_arc(arc_greeter.clone()));
    // let serve_dir: ServeDir = ServeDir::new(&arc_greeter.config.base_dir);

    let app = Router::new().merge(axum_routers).merge(svc.into_router());

    axum::serve(listener, wrap_files_router(arc_greeter, app)).await?;
    Ok(())
}
