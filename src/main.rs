#[macro_use]
extern crate diesel;

#[macro_use]
extern crate validator_derive;

#[macro_use]
extern crate log;

use actix_web::{App, HttpServer};
use env_logger::{Env, Target};

mod controllers;
mod db;
mod models;
mod routes;
mod schema;
mod utils;

use crate::routes::define_routes;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {

    // configure database connection
    let pool = db::create_pool();

    // configure logging
    env_logger::from_env(Env::default().default_filter_or("info"))
        .target(Target::Stdout)
        .init();

    // configure server
    HttpServer::new(move || App::new().data(pool.clone()).configure(define_routes))
        .bind("0.0.0.0:8080")?
        .run()
        .await
}
