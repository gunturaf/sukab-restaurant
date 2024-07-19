use std::{
    env,
    thread::{self, JoinHandle},
};

use rand::Rng;
use serde::{Deserialize, Serialize};

struct UrlBuilder {
    base_url: String,
    table_number: i32,
    order_id: Option<i32>,
}

impl UrlBuilder {
    fn get_base_url() -> String {
        match env::var("SERVER_BASE_URL") {
            Ok(v) => v,
            Err(_) => "http://localhost:8080".to_string(),
        }
    }

    fn new(table_number: i32) -> Self {
        let base_url = UrlBuilder::get_base_url();

        UrlBuilder {
            base_url,
            table_number,
            order_id: None,
        }
    }

    fn order_id(&mut self, order_id: i32) -> &Self {
        self.order_id = Some(order_id);
        self
    }

    fn url(&self) -> String {
        let url = format!("{}/table/{}/order", self.base_url, self.table_number);
        match self.order_id {
            Some(oid) => format!("{}/{}", url, oid),
            None => url,
        }
    }
}

#[derive(Deserialize)]
struct CreateResponse {
    order: Order,
}

#[derive(Deserialize)]
struct Order {
    order_id: i32,
}

#[derive(Serialize)]
struct CreateRequest {
    menu_id: i32,
}

fn get_random_range_inclusive(min: i32, max: i32) -> i32 {
    let mut rr = rand::thread_rng();
    rr.gen_range(min..=max)
}

fn send_create_order(table_number: i32, menu_id: i32) -> Result<i32, ()> {
    let url = UrlBuilder::new(table_number).url();

    let client = reqwest::blocking::ClientBuilder::default().build().unwrap();
    let req_body = CreateRequest { menu_id };

    let response = client.post(url).json(&req_body).send();
    match response {
        Ok(v) => {
            let stat_code = v.status();
            let r: CreateResponse = v.json().unwrap();
            log::info!(
                "create order, status {}, order ID = {}",
                stat_code,
                r.order.order_id.clone()
            );
            Ok(r.order.order_id)
        }
        Err(e) => {
            log::error!("{:?}", e);
            Err(())
        }
    }
}

fn send_delete_order(table_number: i32, order_id: i32) {
    let url = UrlBuilder::new(table_number).order_id(order_id).url();

    let client = reqwest::blocking::ClientBuilder::default().build().unwrap();

    let response = client.delete(url).send();
    match response {
        Ok(v) => {
            log::info!("delete order ID {}, status {:?}", order_id, v.status());
        }
        Err(e) => {
            log::error!("delete order ID {}, failure {:?}", order_id, e);
        }
    }
}

fn send_detail_order(table_number: i32, order_id: i32) {
    let url = UrlBuilder::new(table_number).order_id(order_id).url();

    let response = reqwest::blocking::get(url);
    match response {
        Ok(v) => {
            let stat_code = v.status();
            log::info!(
                "get order detail by ID {}, status {}, response {:?}",
                order_id,
                stat_code,
                v.text().unwrap()
            );
        }
        Err(e) => {
            log::error!("get order detail by ID {}, failure {:?}", order_id, e);
        }
    }
}

fn send_list_orders(table_number: i32) {
    let url = UrlBuilder::new(table_number).url();

    let response = reqwest::blocking::get(url);
    match response {
        Ok(v) => {
            let stat_code = v.status();
            log::info!(
                "list orders by Table Number {}, status = {}, response = {:?}",
                table_number,
                stat_code,
                v.text().unwrap()
            );
        }
        Err(e) => {
            log::error!(
                "list orders by Table Number {}, failure {:?}",
                table_number, e
            );
        }
    }
}

fn send_request(table_number: i32, menu_id: i32) {
    let order_id = send_create_order(table_number, menu_id).unwrap();
    send_list_orders(table_number);
    send_detail_order(table_number, order_id);
    send_delete_order(table_number, order_id)
}

fn get_thread_count(default_count: i32) -> i32 {
    match env::var("CLIENT_THREAD_COUNT") {
        Ok(v) => v.parse().unwrap_or(default_count),
        Err(_) => default_count,
    }
}

fn set_global_logger() {
    let rust_log_flag = "RUST_LOG";
    match env::var(rust_log_flag) {
        Ok(_) => {}
        Err(_) => env::set_var(rust_log_flag, "info"),
    };
    env_logger::init();
}

fn main() {
    set_global_logger();
    let thread_count = get_thread_count(10);
    let mut w = Vec::<JoinHandle<()>>::new();

    for _ in 0..thread_count {
        let handle = thread::spawn(|| {
            let table_number = get_random_range_inclusive(1, 100);
            let menu_id = get_random_range_inclusive(1, 10);
            send_request(table_number, menu_id);
        });
        w.push(handle);
    }

    for v in w {
        v.join().unwrap();
    }
}
