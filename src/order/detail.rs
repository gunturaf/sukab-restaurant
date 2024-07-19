use std::fmt;

use actix_web::{
    body::BoxBody, get, http::StatusCode, web, HttpResponse, HttpResponseBuilder, ResponseError,
};
use serde::{Deserialize, Serialize};

use crate::{
    db::{self, order::Order, OperationError},
    order::InternalServerErrorBody,
};

use super::{BadRequestBody, MenuData, OrderData};

/// The input data to get detail of an Order.
struct Input {
    table_number: u32,
    order_id: u32,
}

impl Input {
    fn new(path_params: PathParams) -> Self {
        Self {
            table_number: path_params.table_number,
            order_id: path_params.order_id,
        }
    }

    /// performs simple request validation to make check some bounds.
    fn validate(self) -> Result<Self, DetailFailure> {
        if self.table_number < 1 || self.table_number > 100 {
            return Err(DetailFailure::InvalidInput(BadRequestBody {
                error: true,
                message: String::from("table_number must be in range of 1 to 100"),
            }));
        }
        return Ok(self);
    }
}

#[derive(Serialize, Deserialize)]
struct PathParams {
    table_number: u32,
    order_id: u32,
}

#[derive(Debug)]
enum DetailFailure {
    InvalidInput(BadRequestBody),
    InternalServerError(OperationError),
}

impl fmt::Display for DetailFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to get order detail")
    }
}

impl ResponseError for DetailFailure {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            DetailFailure::InvalidInput(_) => StatusCode::BAD_REQUEST,
            DetailFailure::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self {
            DetailFailure::InvalidInput(r) => HttpResponseBuilder::new(self.status_code()).json(r),
            DetailFailure::InternalServerError(e) => {
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
    order: OrderData,
}

impl SuccessResponseBody {
    fn new(order: Order) -> Self {
        Self {
            order: OrderData {
                order_id: order.order_id,
                table_number: order.table_number,
                cook_time: order.cook_time,
                menu: MenuData {
                    id: order.menu_id as i64,
                    name: order.name.clone().unwrap_or("".to_string()),
                },
                created_at: OrderData::format_time(order.created_at),
            },
        }
    }
}

#[get("/order/{order_id}")]
async fn handler(
    order_repository: web::Data<dyn db::order::Repository>,
    path_params: web::Path<PathParams>,
) -> Result<HttpResponse, DetailFailure> {
    let input = Input::new(path_params.into_inner()).validate()?;

    let result_data = order_repository
        .get_order_detail(input.table_number as i32, input.order_id as i64)
        .await
        .map_err(|e| DetailFailure::InternalServerError(e))?;

    match result_data {
        Some(order) => Ok(HttpResponse::Ok().json(SuccessResponseBody::new(order))),
        None => Ok(HttpResponse::NotFound().body("".to_string())),
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
    /// when: get order detail.
    /// then: response status code is 400.
    async fn test_invalid_table_id() {
        let table_number = 0;
        let order_id = 1;

        let order_repo = crate::db::order::MockRepository::new();
        let arc_order_repo: Arc<dyn db::order::Repository> = Arc::new(order_repo);

        let app = test::init_service(
            App::new()
                .app_data(Data::from(arc_order_repo))
                .service(web::scope("/table/{table_number}").service(handler)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri(format!("/table/{}/order/{}", table_number, order_id).as_str())
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());
    }

    #[actix_web::test]
    /// given: all request and inputs are valid.
    /// when: get order detail.
    /// then: response status code is 200.
    async fn test_success() {
        let expect_menu_name = "Nasi Goreng".to_string();
        let expect_menu_name_cp = expect_menu_name.clone();
        let expect_order_id = 123;
        let table_number = 3;

        let mut order_repo = crate::db::order::MockRepository::new();
        order_repo
            .expect_get_order_detail()
            .once()
            .returning(move |table_number, order_id| {
                let expect_order_data = Order {
                    order_id,
                    table_number,
                    menu_id: 2,
                    cook_time: 3,
                    name: Some(expect_menu_name_cp.clone()),
                    created_at: OffsetDateTime::now_utc(),
                };
                Ok(Some(expect_order_data))
            });

        let arc_order_repo: Arc<dyn db::order::Repository> = Arc::new(order_repo);

        let app = test::init_service(
            App::new()
                .app_data(Data::from(arc_order_repo))
                .service(web::scope("/table/{table_number}").service(handler)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri(format!("/table/{}/order/{}", table_number, expect_order_id).as_str())
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let response_body: SuccessResponseBody = test::read_body_json(resp).await;
        assert_eq!(response_body.order.table_number, table_number);
        assert_eq!(response_body.order.order_id, expect_order_id);
        assert_eq!(response_body.order.menu.name, expect_menu_name);
        assert_ne!(response_body.order.cook_time, 0);
    }

    #[actix_web::test]
    /// given: failure when accessing the database.
    /// when: get order detail.
    /// then: response status code is 500.
    async fn test_database_failure() {
        let table_number = 3;
        let expect_order_id = 222;

        let mut order_repo = crate::db::order::MockRepository::new();
        order_repo
            .expect_get_order_detail()
            .once()
            .returning(|_, _| Err(OperationError::OtherError));
        let arc_order_repo: Arc<dyn db::order::Repository> = Arc::new(order_repo);

        let app = test::init_service(
            App::new()
                .app_data(Data::from(arc_order_repo))
                .service(web::scope("/table/{table_number}").service(handler)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri(format!("/table/{}/order/{}", table_number, expect_order_id).as_str())
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_server_error());
    }
}
