use actix_web::web;
use serde::{Deserialize, Serialize};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

pub mod create;
pub mod delete;
pub mod detail;
pub mod list;

#[derive(Serialize, Deserialize)]
struct OrderData {
    order_id: i64,
    table_number: i32,
    cook_time: i32,
    menu: MenuData,
    created_at: String,
}

impl OrderData {
    fn format_time(dt: OffsetDateTime) -> String {
        dt.format(&Rfc3339).unwrap_or("---".to_string())
    }
}

#[derive(Serialize, Deserialize)]
struct MenuData {
    id: i64,
    name: String,
}

#[derive(Serialize, Debug)]
struct BadRequestBody {
    error: bool,
    message: String,
}

#[derive(Serialize, Debug)]
struct InternalServerErrorBody {
    error: bool,
    message: String,
}

pub fn service() -> actix_web::Scope {
    web::scope("/table/{table_number}")
        .service(detail::handler)
        .service(create::handler)
        .service(delete::handler)
        .service(list::handler)
}
