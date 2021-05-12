use std::sync::Arc;

use hyper::{Body, Method, Request, Response, StatusCode};

use crate::AppResult;
use crate::db::DbService;
use crate::entities::User;
use crate::errors::AppError;

pub struct Endpoints {
    db_service: DbService,
}

struct Param<'a> {
    name: &'a str,
    value: &'a str,
}

fn parse_param<'a>(pair: &'a str) -> AppResult<Param<'a>> {
    let res: Vec<&'a str> = pair.split("=").collect();
    let &name = res.get(0)
        .ok_or(AppError::new("Invalid query param".to_string()))?;

    let &value = res.get(1)
        .ok_or(AppError::new("Invalid query param".to_string()))?;

    Ok(Param {
        name,
        value,
    })
}

fn unwrap(res: Vec<AppResult<Param>>) -> AppResult<Vec<Param>> {
    let mut new_res = vec![];
    for r in res.into_iter() {
        new_res.push(r?);
    }
    Ok(new_res)
}

impl Endpoints {
    pub fn new(db_service: DbService) -> Self {
        Endpoints {
            db_service
        }
    }

    fn parse_query_params(query: Option<&str>) -> AppResult<Vec<Param>> {
        match query {
            Some(query_str) => {
                let res: Vec<AppResult<Param>> = query_str
                    .split("&")
                    .map(parse_param)
                    .collect();

                unwrap(res)
            }
            _ => Ok(vec![])
        }
    }

    fn get_param_value<'a>(query: Option<&'a str>, param: &'static str) -> AppResult<&'a str> {
        Endpoints::parse_query_params(query)?
            .iter()
            .find(|p| { p.name == param })
            .map(|p| { p.value })
            .ok_or(AppError::new(format!("No {} param specified", param)))
    }

    pub async fn routes(self: Arc<Self>, req: Request<Body>) -> AppResult<Response<Body>> {
        match (req.method(), req.uri().path()) {
            (&Method::GET, "/health") => {
                self.health()
            }
            (&Method::GET, "/users/by-id") => {
                let id = Endpoints::get_param_value(req.uri().query(), "id")?;
                self.get_user_by_id(id).await
            }
            (&Method::GET, "/users/by-name") => {
                let name = Endpoints::get_param_value(req.uri().query(), "name")?;
                self.get_users_by_name(name).await
            }
            (&Method::POST, "/users") => {
                self.add_user(req).await
            }
            _ => {
                Ok(Response::builder().status(StatusCode::NOT_FOUND).body(Body::empty()).unwrap())
            }
        }
    }

    pub async fn routes_with_error_handling(self: Arc<Self>, req: Request<Body>) -> AppResult<Response<Body>> {
        let res = self.routes(req).await;
        match res {
            Ok(response) => Ok(response),
            Err(error) =>
                Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(error.message.into())
                    .unwrap())
        }
    }

    fn health(&self) -> AppResult<Response<Body>> {
        Ok(Response::builder().status(StatusCode::OK).body("healthy".into()).unwrap())
    }

    async fn get_user_by_id(&self, id: &str) -> AppResult<Response<Body>> {
        let user_opt = self.db_service.get_user_by_id(id).await?;
        match user_opt {
            Some(user) => {
                let user_json = serde_json::to_string(&user).unwrap();
                Ok(Response::builder().status(StatusCode::OK).body(user_json.into()).unwrap())
            }
            _ => {
                let body = format!("User {} not found", id);
                Ok(Response::builder().status(StatusCode::NOT_FOUND).body(body.into()).unwrap())
            }
        }
    }

    async fn get_users_by_name(&self, name: &str) -> AppResult<Response<Body>> {
        let users = self.db_service.get_users_by_name(name).await?;
        let users_json = serde_json::to_string(&users).unwrap();
        Ok(Response::builder().status(StatusCode::OK).body(users_json.into()).unwrap())
    }

    async fn add_user(&self, req: Request<Body>) -> AppResult<Response<Body>> {
        let body = hyper::body::to_bytes(req.into_body()).await;
        let body_res = match body {
            Ok(b) => Ok(b),
            Err(_) => Err(AppError::new("Couldn't read request body".to_string()))
        }?;
        let user: User = match serde_json::from_slice(&body_res[..]) {
            Ok(user) => Ok(user),
            Err(e) => Err(AppError::new(e.to_string()))
        }?;
        let _ = self.db_service.insert_user(&user).await?;
        Ok(
            Response::builder()
                .status(StatusCode::OK)
                .body(serde_json::to_string(&user).unwrap().into())
                .unwrap()
        )
    }
}