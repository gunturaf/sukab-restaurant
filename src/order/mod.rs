use actix_web::web;

pub mod create;

pub fn service() -> actix_web::Scope {
    web::scope("/table/{table_number}").service(create::handler)
}
