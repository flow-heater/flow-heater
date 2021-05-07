use crate::configuration::Settings;
use crate::routes::{admin, conversation, execute, health_check};
use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use fh_db::ReqCmd;
use fh_v8::ProcessorCmd;
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
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(&address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(listener, configuration.application.base_url, tx_db, tx_v8)?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub struct ApplicationBaseUrl(pub String);

fn run(
    listener: TcpListener,
    base_url: String,
    tx_db: mpsc::Sender<ReqCmd>,
    tx_v8: mpsc::Sender<ProcessorCmd>,
) -> Result<Server, std::io::Error> {
    let tx_db = Data::new(tx_db);
    let tx_v8 = Data::new(tx_v8);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .service(
                web::scope("/admin")
                    .service(
                        web::resource("/processor").route(web::post().to(admin::create_processor)),
                    )
                    .service(
                        web::resource("/processor/{id}")
                            .route(web::get().to(admin::get_processor))
                            .route(web::put().to(admin::update_processor))
                            .route(web::delete().to(admin::delete_processor)),
                    ),
            )
            .service(
                web::scope("/conversation")
                    .service(
                        web::resource("/{id}")
                            .route(web::get().to(conversation::get_request_conversation)),
                    )
                    .service(
                        web::resource("/{id}/audit_item").route(
                            web::get().to(conversation::get_request_conversation_audit_items),
                        ),
                    ),
            )
            .service(
                web::scope("/processor/{id}")
                    .service(web::resource("/run").to(execute::run_processor))
                    .service(
                        web::resource("/run_with_prelude").to(execute::run_processor_with_prelude),
                    ),
            )
            .service(web::resource("/hello/{name}").to(execute::deprecated_run_processor))
            .route("/health_check", web::get().to(health_check))
            .app_data(tx_db.clone())
            .app_data(tx_v8.clone())
            .data(ApplicationBaseUrl(base_url.clone()))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
