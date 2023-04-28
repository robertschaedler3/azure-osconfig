use std::{
    fs,
    path::Path,
    sync::{Arc, Mutex},
};

use anyhow::{Error, Result};
use hyper::{Body, Request, Response, Server, StatusCode};
use hyperlocal::UnixServerExt;

use routerify::{prelude::RequestExt, Middleware, RequestInfo, Router};
use routerify_unixsocket::UnixRouterService;

use platform_v2::{platform::Platform, Value};

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let path = Path::new("/run/osconfig/mpid.sock");
    if path.exists() {
        fs::remove_file(path).unwrap();
    } else {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
    }

    let platform = Platform::load()?;
    let platform = Arc::new(Mutex::new(platform));

    let router: Router<Body, Error> = Router::builder()
        .data(platform.clone())
        .middleware(Middleware::pre(logger))
        .get("/:component/:object", reported_handler)
        .post("/:component/:object", desired_handler)
        .err_handler_with_info(error_handler)
        .build()
        .unwrap();

    let service = UnixRouterService::new(router).unwrap();
    let server = Server::bind_unix(path)?.serve(service);

    server.await?;

    Ok(())
}

// A handler for POST "/:component/:object" requests
async fn desired_handler(mut req: Request<Body>) -> Result<Response<Body>> {
    let value = parse_body::<Value>(&mut req).await.unwrap();
    let platform = req.data::<Arc<Mutex<Platform>>>().unwrap();

    let component = req.param("component").unwrap();
    let object = req.param("object").unwrap();

    platform.lock().unwrap().set(&component, &object, &value)?;

    Ok(Response::new(Body::from("")))
}

// A handler for GET "/:component/:object" requests
async fn reported_handler(req: Request<Body>) -> Result<Response<Body>> {
    let platform = req.data::<Arc<Mutex<Platform>>>().unwrap();

    let component = req.param("component").unwrap();
    let object = req.param("object").unwrap();

    let value = platform.lock().unwrap().get(&component, &object)?;

    Ok(Response::new(Body::from(serde_json::to_string(&value)?)))
}

async fn logger(req: Request<Body>) -> Result<Request<Body>> {
    println!(
        "{} {} {}",
        req.remote_addr(),
        req.method(),
        req.uri().path()
    );
    Ok(req)
}

async fn error_handler(err: routerify::RouteError, _: RequestInfo) -> Response<Body> {
    eprintln!("{}", err);
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from(format!("Error: {}", err)))
        .unwrap()
}

async fn parse_body<T>(req: &mut Request<Body>) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let body = hyper::body::to_bytes(req.body_mut()).await?;
    let body = String::from_utf8(body.to_vec())?;
    let body = serde_json::from_str(&body)?;
    Ok(body)
}
