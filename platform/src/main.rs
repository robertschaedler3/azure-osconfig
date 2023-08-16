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

use platform::{platform::Platform, Value};

#[tokio::main]
async fn main() -> Result<()> {
    osc::init_logger();

    // Get a path from the command line
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        println!("Usage: platform <path>");
        std::process::exit(1);
    });

    let path = Path::new(&path).canonicalize()?;

    let library = platform::module::Library::load(path)?;

    let info = library.info("blah")?;

    println!("{:#?}", info);

    let session = library.open("Test", 0)?;

    let payload = r#"{"message": "hello foo"}"#;
    library.set(&session, "Blah", "foo", payload, payload.len())?;

    library.set(&session, "Blah", "bar", payload, payload.len())?;
    library.set(&session, "Blah", "bar", payload, payload.len())?;

    library.close(session)?;

    // let path = Path::new("/run/osconfig/mpid.sock");
    // if path.exists() {
    //     fs::remove_file(path).unwrap();
    // } else {
    //     fs::create_dir_all(path.parent().unwrap()).unwrap();
    // }

    // // TODO: middleware for loading module clients as needed and injecting the correct client into each call ?
    // let platform = Platform::load()?;
    // let platform = Arc::new(Mutex::new(platform));

    // let router: Router<Body, Error> = Router::builder()
    //     .data(platform.clone())
    //     .middleware(Middleware::pre(logger))
    //     .get("/:component/:object", reported_handler)
    //     .post("/:component/:object", desired_handler)
    //     .err_handler_with_info(error_handler)
    //     .build()
    //     .unwrap();

    // let service = UnixRouterService::new(router).unwrap();
    // let server = Server::bind_unix(path)?.serve(service);

    // server.await?;

    Ok(())
}

// TODO: figure out how to "augment" get/set requests all the way to the modules
// so that multiple properties can be changed in a single "round trip"
//
// Ideally this will work by passing some "executor" to the modules that will
// orchestrate the property resolution, rather than requiring the modules to
// re-implment this functionality and "be aware" of the platform's internal
// workings/orchestration.
//
// This will also allow the platform to "batch" multiple property changes into
// a single "round trip" to the modules, which will be more efficient.
//
// Each module will "register" the properties (desired/reported) that it supports,
// along with a "handler" that will be called when the property is set/get.
//
// Modules will allow for an addional "context" (provided by MmiOpen or similar)
// parameter to be passed to each handler that will store an internal "state"
// for the module. This will allow the module to "cache" the property value
// or perform other optimizations.
//
// The implmentation of a module should abstract away the things like
// lifecycle management, property resolution, etc. so that the module
// can focus on the "business logic" of resolving each individual property.
//
// Rather than interacting with each shared-object directly, the platform
// will spawn a ModuleClient process for each modules so that failures
// are iscolated. This will also allow the platform to "restart" a module
// if it crashes and manage the lifecycle of each module more effectively.
//
// Nice to have:
// - The ability to return information from each get/set operation
//   (i.e. rich error messages)
//

//REVIEW: how to make this generic enough to query multiple properties in one "round trip"
// A handler for POST "/:component/:object" requests
async fn desired_handler(mut req: Request<Body>) -> Result<Response<Body>> {
    let value = parse_body::<Value>(&mut req).await.unwrap();
    let platform = req.data::<Arc<Mutex<Platform>>>().unwrap();

    let component = req.param("component").unwrap();
    let object = req.param("object").unwrap();

    platform.lock().unwrap().set(&component, &object, &value)?;

    Ok(Response::new(Body::from("")))
}

//REVIEW: how to make this generic enough to query multiple properties in one "round trip"
// A handler for GET "/:component/:object" requests
async fn reported_handler(req: Request<Body>) -> Result<Response<Body>> {
    let platform = req.data::<Arc<Mutex<Platform>>>().unwrap();

    let component = req.param("component").unwrap();
    let object = req.param("object").unwrap();

    let value = platform.lock().unwrap().get(&component, &object)?;

    Ok(Response::new(Body::from(serde_json::to_string(&value)?)))
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

async fn parse_body<T>(req: &mut Request<Body>) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let body = hyper::body::to_bytes(req.body_mut()).await?;
    let body = String::from_utf8(body.to_vec())?;
    let body = serde_json::from_str(&body)?;
    Ok(body)
}
