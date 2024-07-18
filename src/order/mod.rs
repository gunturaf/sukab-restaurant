use actix_web::web;
use serde::{Deserialize, Serialize};

pub mod create;
pub mod list;
pub mod detail;

#[derive(Serialize, Deserialize)]
struct OrderData {
    order_id: i64,
    table_number: i32,
    cook_time: i32,
    menu: MenuData,
    created_at: String,
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
        .service(list::handler)
}
