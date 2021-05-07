use crate::configuration::{DatabaseSettings, Settings};
use crate::routes::{admin, health_check};
use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use fh_db::ReqCmd;
use fh_v8::ProcessorCmd;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::TcpListener;
use tokio::sync::mpsc;
use tracing_actix_web::TracingLogger;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(
        configuration: Settings,
        tx_db: mpsc::Sender<ReqCmd>,
        tx_v8: mpsc::Sender<ProcessorCmd>,
    ) -> Result<Self, std::io::Error> {
        let db_pool = get_connection_pool(&configuration.database)
            .await
            .expect("Failed to connect to Postgres.");

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(&address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener,
            db_pool,
            configuration.application.base_url,
            tx_db,
            tx_v8,
        )?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub async fn get_connection_pool(configuration: &DatabaseSettings) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_with(configuration.with_db())
        .await
}

pub struct ApplicationBaseUrl(pub String);

fn run(
    listener: TcpListener,
    db_pool: PgPool,
    base_url: String,
    tx_db: mpsc::Sender<ReqCmd>,
    tx_v8: mpsc::Sender<ProcessorCmd>,
) -> Result<Server, std::io::Error> {
    let db_pool = Data::new(db_pool);
    let tx_db = Data::new(tx_db);
    let tx_v8 = Data::new(tx_v8);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/admin/processor", web::post().to(admin::create_processor))
            .app_data(db_pool.clone())
            .app_data(tx_db.clone())
            .app_data(tx_v8.clone())
            .data(ApplicationBaseUrl(base_url.clone()))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
