use actix_web::web;

pub mod create;
pub mod list;

pub fn service() -> actix_web::Scope {
    web::scope("/table/{table_number}")
        .service(create::handler)
        .service(list::handler)
}
