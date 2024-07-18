use std::fmt;

use actix_web::{
    body::BoxBody, get, http::StatusCode, web, HttpResponse, HttpResponseBuilder, ResponseError,
};
use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;

use crate::db::{self, order::Order, OperationError};

/// The input data to create a new Order which came from the User.
struct Input {
    table_number: u32,
}

impl Input {
    fn new(table_number: u32) -> Self {
        Self { table_number }
    }

    /// performs simple request validation to make check some bounds.
    fn validate(&self) -> Result<bool, CreateFailure> {
        if self.table_number < 1 || self.table_number > 100 {
            return Err(CreateFailure::InvalidInput(BadRequestBody {
                error: true,
                message: String::from("table_number must be in range of 1 to 100"),
            }));
        }
        return Ok(true);
    }
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

#[derive(Serialize, Deserialize)]
struct PathParams {
    table_number: u32,
}

#[derive(Debug)]
enum CreateFailure {
    InvalidInput(BadRequestBody),
    InternalServerError(OperationError),
}

impl fmt::Display for CreateFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to create order")
    }
}

impl ResponseError for CreateFailure {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            CreateFailure::InvalidInput(_) => StatusCode::BAD_REQUEST,
            CreateFailure::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self {
            CreateFailure::InvalidInput(r) => HttpResponseBuilder::new(self.status_code()).json(r),
            CreateFailure::InternalServerError(e) => {
                log::error!("{:?}", e);
                HttpResponseBuilder::new(self.status_code()).json(InternalServerErrorBody {
                    error: true,
                    message: "An unknown server error has occurred, please try again later."
                        .to_string(),
                })
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct SuccessResponseBody {
    orders: Vec<OrderData>,
}

#[derive(Serialize, Deserialize)]
struct OrderData {
    order_id: i64,
    table_number: i32,
    menu_id: i32,
    cook_time: i32,
    name: String,
    created_at: String,
}

impl SuccessResponseBody {
    fn new(orders: Vec<Order>) -> Self {
        let order_list: Vec<OrderData> = orders
            .iter()
            .map(|order| OrderData {
                order_id: order.order_id,
                table_number: order.table_number,
                menu_id: order.menu_id,
                cook_time: order.cook_time,
                name: order.name.clone().unwrap_or("".to_string()),
                created_at: order.created_at.format(&Rfc3339).unwrap_or("".to_string()),
            })
            .collect();
        Self { orders: order_list }
    }
}

#[get("/order")]
async fn handler(
    order_repository: web::Data<dyn db::order::Repository>,
    path_params: web::Path<PathParams>,
) -> Result<HttpResponse, CreateFailure> {
    let input = Input::new(path_params.table_number);
    input.validate()?;

    match order_repository
        .list_by_table(input.table_number as i32)
        .await
    {
        Ok(orders) => Ok(HttpResponse::Ok().json(SuccessResponseBody::new(orders))),
        Err(e) => Err(CreateFailure::InternalServerError(e)),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use actix_web::{test, App};
    use time::OffsetDateTime;
    use web::Data;

    use super::*;

    #[actix_web::test]
    /// given: zero table_id.
    /// when: list Orders in a Table.
    /// then: response status code is 400.
    async fn test_invalid_table_id() {
        let table_number = 0;

        let order_repo = crate::db::order::MockRepository::new();
        let arc_order_repo: Arc<dyn db::order::Repository> = Arc::new(order_repo);

        let app = test::init_service(
            App::new()
                .app_data(Data::from(arc_order_repo))
                .service(web::scope("/table/{table_number}").service(handler)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri(format!("/table/{}/order", table_number).as_str())
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());
    }

    #[actix_web::test]
    /// given: all request and inputs are valid.
    /// when: list Orders in a Table.
    /// then: response status code is 200.
    async fn test_success() {
        let expect_menu_name = "Nasi Goreng".to_string();
        let expect_menu_name_cp = expect_menu_name.clone();
        let expect_order_id = 123;
        let table_number = 3;

        let mut order_repo = crate::db::order::MockRepository::new();
        order_repo
            .expect_list_by_table()
            .once()
            .returning(move |table_number| {
                let expect_order_data = Order {
                    order_id: expect_order_id,
                    table_number,
                    menu_id: 2,
                    cook_time: 3,
                    name: Some(expect_menu_name_cp.clone()),
                    created_at: OffsetDateTime::now_utc(),
                };
                Ok(vec![expect_order_data])
            });

        let arc_order_repo: Arc<dyn db::order::Repository> = Arc::new(order_repo);

        let app = test::init_service(
            App::new()
                .app_data(Data::from(arc_order_repo))
                .service(web::scope("/table/{table_number}").service(handler)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri(format!("/table/{}/order", table_number).as_str())
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let response_body: SuccessResponseBody = test::read_body_json(resp).await;
        assert_eq!(response_body.orders[0].table_number, table_number);
        assert_eq!(response_body.orders[0].order_id, expect_order_id);
        assert_eq!(response_body.orders[0].name, expect_menu_name);
        assert_ne!(response_body.orders[0].cook_time, 0);
    }

    #[actix_web::test]
    /// given: zero table_id.
    /// when: list Orders in a Table.
    /// then: response status code is 500.
    async fn test_database_failure() {
        let table_number = 3;

        let mut order_repo = crate::db::order::MockRepository::new();
        order_repo
            .expect_list_by_table()
            .once()
            .returning(|_| Err(OperationError::OtherError));
        let arc_order_repo: Arc<dyn db::order::Repository> = Arc::new(order_repo);

        let app = test::init_service(
            App::new()
                .app_data(Data::from(arc_order_repo))
                .service(web::scope("/table/{table_number}").service(handler)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri(format!("/table/{}/order", table_number).as_str())
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_server_error());
    }
}
