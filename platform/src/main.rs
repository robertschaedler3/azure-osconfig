use anyhow::{Context, Error, Result};
use hyper::{Body, Request, Response, Server, StatusCode};
use hyperlocal::UnixServerExt;
use routerify::{prelude::RequestExt, Middleware, RequestInfo, Router};
use routerify_unixsocket::UnixRouterService;

use platform::Platform;

const SOCKET_PATH: &str = "/var/run/osc-platform.sock";

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let path = std::path::Path::new(SOCKET_PATH);

    if path.exists() {
        log::debug!("Removing existing socket file: {:?}", path);
        std::fs::remove_file(path)?;
    } else {
        let parent = path.parent().context("Failed to get parent directory")?;
        std::fs::create_dir_all(parent)?;
    }

    let platform = Platform {};

    let router: Router<Body, Error> = Router::builder()
        .data(platform.clone())
        .middleware(Middleware::pre(logger))
        .get("/read", read_handler)
        .post("/write", write_handler)
        .err_handler_with_info(error_handler)
        .build()
        .unwrap();

    let router = UnixRouterService::new(router).unwrap();
    let server = Server::bind_unix(path)?.serve(router);

    server.await?;

    Ok(())
}

async fn write_handler(mut req: Request<Body>) -> Result<Response<Body>> {
    Ok(Response::new(Body::from("TODO: implement write_handler")))
}

async fn read_handler(req: Request<Body>) -> Result<Response<Body>> {
    Ok(Response::new(Body::from("TODO: implement read_handler")))
}

async fn logger(req: Request<Body>) -> Result<Request<Body>> {
    log::trace!(
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
