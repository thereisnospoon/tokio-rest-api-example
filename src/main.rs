mod db;
mod entities;
mod routes;
mod errors;

use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use hyper::Server;
use hyper::service::{make_service_fn, service_fn};

use crate::db::DbService;
use crate::routes::Endpoints;
use crate::errors::AppError;

type AppResult<T> =  Result<T, AppError>;

#[tokio::main]
async fn main() {

    let db_service = DbService::new();
    db_service.init_db().await.unwrap();

    let endpoints = Arc::new(Endpoints::new(db_service));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let make_svc = make_service_fn( move |_conn| {
        let endpoints = endpoints.clone();
        async move {
            let endpoints = endpoints.clone();
            Ok::<_, Infallible>(service_fn(move |req| {
                endpoints.clone().routes_with_error_handling(req)
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    };
}
