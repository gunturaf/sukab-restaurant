use std::fmt;

use actix_web::{
    body::BoxBody, delete, http::StatusCode, web, HttpResponse, HttpResponseBuilder, ResponseError,
};
use serde::{Deserialize, Serialize};

use crate::{
    db::{self, OperationError},
    order::InternalServerErrorBody,
};

use super::BadRequestBody;

/// The input data to get detail of an Order.
struct Input {
    table_number: u32,
    order_id: u32,
}

impl Input {
    fn new(table_number: u32, order_id: u32) -> Self {
        Self {
            table_number,
            order_id,
        }
    }

    /// performs simple request validation to make check some bounds.
    fn validate(&self) -> Result<bool, DetailFailure> {
        if self.table_number < 1 || self.table_number > 100 {
            return Err(DetailFailure::InvalidInput(BadRequestBody {
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
    order_id: i64,
}

#[delete("/order/{order_id}")]
async fn handler(
    order_repository: web::Data<dyn db::order::Repository>,
    path_params: web::Path<PathParams>,
) -> Result<HttpResponse, DetailFailure> {
    let input = Input::new(path_params.table_number, path_params.order_id);
    input.validate()?;

    let result_data = order_repository
        .delete_order(input.table_number as i32, input.order_id as i64)
        .await
        .map_err(|e| DetailFailure::InternalServerError(e))?;

    match result_data {
        Some(order_id) => Ok(HttpResponse::Ok().json(SuccessResponseBody { order_id })),
        None => Ok(HttpResponse::NotFound().body("".to_string())),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use actix_web::{test, App};
    use web::Data;

    use super::*;

    #[actix_web::test]
    /// given: zero table_id.
    /// when: delete an order.
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

        let req = test::TestRequest::delete()
            .uri(format!("/table/{}/order/{}", table_number, order_id).as_str())
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());
    }

    #[actix_web::test]
    /// given: all request and inputs are valid.
    /// when: delete an order.
    /// then: response status code is 200.
    async fn test_success() {
        let expect_order_id = 123;
        let table_number = 3;

        let mut order_repo = crate::db::order::MockRepository::new();
        order_repo
            .expect_delete_order()
            .once()
            .returning(move |_, order_id| {
                Ok(Some(order_id))
            });

        let arc_order_repo: Arc<dyn db::order::Repository> = Arc::new(order_repo);

        let app = test::init_service(
            App::new()
                .app_data(Data::from(arc_order_repo))
                .service(web::scope("/table/{table_number}").service(handler)),
        )
        .await;

        let req = test::TestRequest::delete()
            .uri(format!("/table/{}/order/{}", table_number, expect_order_id).as_str())
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let response_body: SuccessResponseBody = test::read_body_json(resp).await;
        assert_eq!(response_body.order_id, expect_order_id);
    }

    #[actix_web::test]
    /// given: failure when accessing the database.
    /// when: delete an order.
    /// then: response status code is 500.
    async fn test_database_failure() {
        let table_number = 3;
        let expect_order_id = 222;

        let mut order_repo = crate::db::order::MockRepository::new();
        order_repo
            .expect_delete_order()
            .once()
            .returning(|_, _| Err(OperationError::OtherError));
        let arc_order_repo: Arc<dyn db::order::Repository> = Arc::new(order_repo);

        let app = test::init_service(
            App::new()
                .app_data(Data::from(arc_order_repo))
                .service(web::scope("/table/{table_number}").service(handler)),
        )
        .await;

        let req = test::TestRequest::delete()
            .uri(format!("/table/{}/order/{}", table_number, expect_order_id).as_str())
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_server_error());
    }
}
