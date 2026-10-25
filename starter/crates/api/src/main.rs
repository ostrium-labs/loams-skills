use std::sync::Arc;

use actix_web::{App, HttpServer, web};
use anyhow::Context;
use api::{AppState, configure};
use platform::{Config, Platform};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let config = Config::from_env().context("invalid configuration")?;
    let platform = Platform::connect(&config)
        .await
        .context("connecting to Loams")?;
    let platform = Arc::new(platform);
    let addr = config.bind_addr;

    log::info!("listening on {addr}");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                platform: platform.clone(),
            }))
            .configure(configure)
            .wrap(actix_web::middleware::Logger::default())
    })
    .bind(addr)?
    .run()
    .await
    .context("serving")
}
