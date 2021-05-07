use fh_http::configuration::get_configuration;
use fh_http::startup::Application;
use fh_http::telemetry::{get_subscriber, init_subscriber};

use anyhow::Result;
use fh_db::request_manager;
use fh_v8::request_processing_manager;
use tokio::sync::mpsc;

#[actix_web::main]
async fn main() -> Result<()> {
    let subscriber = get_subscriber("fh-http".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");

    let (tx_db, mut rx_db) = mpsc::channel(4096);
    let (tx_v8, mut rx_v8) = mpsc::channel(4096);
    let application = Application::build(configuration, tx_db, tx_v8).await?;

    let (app, db, v8) = tokio::join!(
        application.run_until_stopped(),
        request_manager(&mut rx_db),
        request_processing_manager(&mut rx_v8)
    );

    app?;
    db?;
    v8?;

    Ok(())
}
