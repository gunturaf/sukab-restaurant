use std::fmt;

use actix_web::{post, web, HttpResponse, Responder};
use rand::Rng;
use serde::Deserialize;

const COOK_DUR_MIN: u16 = 5;
const COOK_DUR_MAX: u16 = 10;

#[derive(Deserialize)]
pub struct RequestBody {
    menu_id: u32,
}

fn new_input(rb: RequestBody, table_number: u32) -> Input {
    Input {
        table_number,
        cook_dur: 0,
        menu_id: rb.menu_id,
    }
    .set_cook_dur()
}

struct Input {
    table_number: u32,
    menu_id: u32,
    cook_dur: u16,
}

impl Input {
    fn set_cook_dur(self) -> Self {
        // TODO: extract rng so that it would be reusable/one time init only?
        let mut rr = rand::thread_rng();
        // get random cook duration, from 5 to 10, inclusive.
        Self {
            cook_dur: rr.gen_range(COOK_DUR_MIN..=COOK_DUR_MAX),
            ..self
        }
    }
}

impl fmt::Display for Input {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Table {}, Menu {}, Cook Duration {}",
            self.table_number, self.menu_id, self.cook_dur
        )
    }
}

#[post("/create")]
async fn handler(
    path_params: web::Path<(u32,)>,
    request_body: web::Json<RequestBody>,
) -> impl Responder {
    let (table_number,) = path_params.into_inner();
    let json_request = request_body.into_inner();
    let input = new_input(json_request, table_number);
    HttpResponse::Ok().body(format!("{} =>> ", input))
}
