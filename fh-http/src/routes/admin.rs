use actix_web::{web, HttpResponse};
use fh_db::request_processor::RequestProcessor;

pub async fn create_processor(body: web::Json<RequestProcessor>) -> HttpResponse {
    println!("{:?}", body);
    HttpResponse::Ok().finish()
}
