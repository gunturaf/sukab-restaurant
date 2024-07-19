use std::fmt;

use actix_web::{
    body::BoxBody, get, http::StatusCode, web, HttpResponse, HttpResponseBuilder, ResponseError,
};
use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;

use crate::{
    db::{self, order::Order, OperationError},
    order::InternalServerErrorBody,
};

use super::{BadRequestBody, MenuData, OrderData};

/// The input data to list Orders.
struct Input {
    table_number: u32,
    page: i32,
    limit: i32,
}

impl Input {
    fn new(path_params: PathParams, query_params: QueryParams) -> Self {
        let table_number = path_params.table_number;
        let page = query_params.page.unwrap_or(0) as i32;
        let limit = query_params
            .limit
            .map(|v| if v == 0 { 1 } else { v })
            .unwrap_or(5) as i32;
        Self {
            table_number,
            page,
            limit,
        }
    }

    /// performs simple request validation to make check some bounds.
    fn validate(&self) -> Result<bool, ListFailure> {
        if self.table_number < 1 || self.table_number > 100 {
            return Err(ListFailure::InvalidInput(BadRequestBody {
                error: true,
                message: String::from("table_number must be in range of 1 to 100"),
            }));
        }
        return Ok(true);
    }
}

#[derive(Serialize, Deserialize)]
struct PathParams {
    table_number: u32,
}

#[derive(Serialize, Deserialize)]
struct QueryParams {
    limit: Option<u32>,
    page: Option<u32>,
}

#[derive(Debug)]
enum ListFailure {
    InvalidInput(BadRequestBody),
    InternalServerError(OperationError),
}

impl fmt::Display for ListFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to list orders")
    }
}

impl ResponseError for ListFailure {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            ListFailure::InvalidInput(_) => StatusCode::BAD_REQUEST,
            ListFailure::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self {
            ListFailure::InvalidInput(r) => HttpResponseBuilder::new(self.status_code()).json(r),
            ListFailure::InternalServerError(e) => {
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

impl SuccessResponseBody {
    fn new(orders: Vec<Order>) -> Self {
        let order_list: Vec<OrderData> = orders
            .iter()
            .map(|order| OrderData {
                order_id: order.order_id,
                table_number: order.table_number,
                menu: MenuData {
                    id: order.menu_id as i64,
                    name: order.name.clone().unwrap_or("".to_string()),
                },
                cook_time: order.cook_time,
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
    query_params: web::Query<QueryParams>,
) -> Result<HttpResponse, ListFailure> {
    let input = Input::new(path_params.into_inner(), query_params.into_inner());
    input.validate()?;

    match order_repository
        .list_by_table(
            input.table_number as i32,
            input.page as i64,
            input.limit as i64,
        )
        .await
    {
        Ok(orders) => Ok(HttpResponse::Ok().json(SuccessResponseBody::new(orders))),
        Err(e) => Err(ListFailure::InternalServerError(e)),
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
            .returning(move |table_number, _, _| {
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
        assert_eq!(response_body.orders[0].menu.name, expect_menu_name);
        assert_ne!(response_body.orders[0].cook_time, 0);
    }

    #[actix_web::test]
    /// given: failure when accessing the database.
    /// when: list Orders in a Table.
    /// then: response status code is 500.
    async fn test_database_failure() {
        let table_number = 3;

        let mut order_repo = crate::db::order::MockRepository::new();
        order_repo
            .expect_list_by_table()
            .once()
            .returning(|_, _, _| Err(OperationError::OtherError));
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
