use std::{env, fmt};

use actix_web::{
    body::BoxBody, http::StatusCode, post, web, HttpResponse, HttpResponseBuilder, ResponseError,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;

use crate::db::{
    self,
    menu::Menu,
    order::Order, OperationError,
};

/// Represents the lower and upper bounds for randomized cook time.
enum CookTimeBounds {
    Min,
    Max,
}

impl CookTimeBounds {
    /// returns the environment variable key to look for.
    fn env_key(&self) -> String {
        match &self {
            Self::Min => String::from("COOK_TIME_MIN"),
            Self::Max => String::from("COOK_TIME_MAX"),
        }
    }
    /// the default values.
    fn default_value(&self) -> u16 {
        match &self {
            Self::Min => 5,
            Self::Max => 10,
        }
    }
    /// returns bounds from environment variables, or defer to predefined default.
    fn get_or_default(&self) -> u16 {
        match env::var(self.env_key()).ok() {
            Some(v) => v.parse().unwrap_or(self.default_value()),
            None => self.default_value(),
        }
    }
}

#[derive(Deserialize)]
pub struct RequestBody {
    menu_id: u32,
}

struct CookTime {
    min: u16,
    max: u16,
}

impl CookTime {
    fn get_random(self) -> u16 {
        let mut rr = rand::thread_rng();
        rr.gen_range(self.min..=self.max)
    }

    fn new() -> Self {
        Self {
            min: CookTimeBounds::Min.get_or_default(),
            max: CookTimeBounds::Max.get_or_default(),
        }
    }
}

/// The input data to create a new Order which came from the User.
struct Input {
    table_number: u32,
    menu_id: u32,
    cook_time: u16,
}

impl fmt::Display for Input {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Table {}, Menu {}, Cook Time {}",
            self.table_number, self.menu_id, self.cook_time
        )
    }
}

impl Input {
    fn new(rb: RequestBody, table_number: u32, cook_time: CookTime) -> Self {
        Self {
            table_number,
            cook_time: cook_time.get_random(),
            menu_id: rb.menu_id,
        }
    }

    /// performs simple request validation to make check some bounds.
    fn validate(&self) -> Result<bool, CreateFailure> {
        if self.menu_id < 1 || self.menu_id > 10 {
            return Err(CreateFailure::InvalidInput(BadRequestBody {
                error: true,
                message: String::from("menu_id must be in range of 5 to 10"),
            }));
        }
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

#[derive(Serialize)]
struct SuccessResponseBody {
    order_id: i64,
    table_number: i32,
    menu_name: String,
    cook_time: i32,
    created_at: String,
}

impl SuccessResponseBody {
    fn new(order: Order, menu: Menu) -> Self {
        Self {
            order_id: order.order_id,
            table_number: order.table_number,
            menu_name: menu.name,
            cook_time: order.cook_time,
            created_at: order
                .created_at
                .format(&Rfc3339)
                .unwrap_or("---".to_string()),
        }
    }
}

#[post("/order")]
async fn handler(
    order_repository: web::Data<dyn db::order::Repository>,
    menu_repository: web::Data<dyn db::menu::Repository>,
    path_params: web::Path<PathParams>,
    request_body: web::Json<RequestBody>,
) -> Result<HttpResponse, CreateFailure> {
    let json_request = request_body.into_inner();
    let cook_time = CookTime::new();
    let input = Input::new(json_request, path_params.table_number, cook_time);
    input.validate()?;

    let order_entity = db::order::Order::new(
        input.table_number as i32,
        input.menu_id as i32,
        input.cook_time as i32,
    );
    match order_repository.create_order(order_entity).await {
        Ok(order_data) => match menu_repository.get_by_id(order_data.menu_id as i64).await {
            Ok(menu) => {
                let response_body = SuccessResponseBody::new(order_data, menu);
                Ok(HttpResponse::Ok().json(response_body))
            }
            Err(e) => {
                log::error!("{:?}", e);
                Err(CreateFailure::InternalServerError(e))
            }
        },
        Err(e) => {
            log::error!("{:?}", e);
            Err(CreateFailure::InternalServerError(e))
        }
    }
}
