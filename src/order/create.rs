use std::{env, fmt};

use actix_web::{post, web, HttpResponse, Responder};
use rand::Rng;
use serde::{Deserialize, Serialize};

enum CookTimeBounds {
    Min,
    Max,
}

impl CookTimeBounds {
    fn env_key(&self) -> String {
        match &self {
            Self::Min => String::from("COOK_TIME_MIN"),
            Self::Max => String::from("COOK_TIME_MAX"),
        }
    }
    fn default_value(&self) -> u16 {
        match &self {
            Self::Min => 5,
            Self::Max => 10,
        }
    }
    fn get_or_default(&self) -> u16 {
        match env::var(self.env_key()) {
            Ok(v) => v.parse().unwrap_or_else(|_| self.default_value()),
            _ => self.default_value(),
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

    fn validate(&self) -> Result<bool, BadRequestBody> {
        if self.menu_id == 0 {
            return Err(BadRequestBody {
                error: true,
                message: String::from("menu_id cannot be zero"),
            });
        }
        if self.table_number == 0 {
            return Err(BadRequestBody {
                error: true,
                message: String::from("table_number cannot be zero"),
            });
        }
        return Ok(true);
    }
}

#[derive(Serialize)]
struct BadRequestBody {
    error: bool,
    message: String,
}

#[post("/order")]
async fn handler(
    path_params: web::Path<(u32,)>,
    request_body: web::Json<RequestBody>,
) -> impl Responder {
    let (table_number,) = path_params.into_inner();
    let json_request = request_body.into_inner();
    let cook_time = CookTime::new();
    let input = Input::new(json_request, table_number, cook_time);
    match input.validate() {
        Err(error_message) => return HttpResponse::BadRequest().json(error_message),
        _ => (),
    }
    HttpResponse::Ok().body(format!("{} =>> ", input))
}
