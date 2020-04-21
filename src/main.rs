// needed for the musl docker build
extern crate openssl;

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate validator_derive;

#[macro_use]
extern crate log;

use actix_cors::Cors;
use actix_web::{App, HttpServer, http::header};
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
    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::new()
                    .allowed_origin("http://localhost:3000")
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .max_age(3600)
                    .finish(),
            )
            .data(pool.clone())
            .configure(define_routes)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
