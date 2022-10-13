use std::{
    fs,
    path::Path,
    sync::{Arc, Mutex},
};

use hyper::{Body, Request, Response, Server};
use hyperlocal::UnixServerExt;
use platform::Platform;
use routerify::{prelude::RequestExt, Error, Router};
use routerify_unixsocket::UnixRouterService;
use serde::Deserialize;

mod platform;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct GetRequest {
    // client_session: String,
    component_name: String,
    object_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SetRequest {
    // client_session: String,
    component_name: String,
    object_name: String,
    payload: osc::module::schema::Value, // TODO: harden this
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let path = Path::new("/run/osconfig/mpid.sock");
    if path.exists() {
        fs::remove_file(path).unwrap();
    } else {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
    }

    let platform = Arc::new(Mutex::new(platform::Platform::load()?));

    let router: Router<Body, Error> = Router::builder()
        .data(platform.clone())
        .post("/MpiGet", |mut req| async move {
            let body = parse_body::<GetRequest>(&mut req).await.unwrap();
            let platform = req.data::<Arc<Mutex<Platform>>>().unwrap();

            log::info!("MpiGet: {:?}", body);

            let component = body.component_name;
            let object = body.object_name;

            let payload = platform.lock().unwrap().get(&component, &object);

            match payload {
                Ok(payload) => Ok(Response::builder()
                    .status(200)
                    .body(Body::from(payload))
                    .unwrap()),
                    Err(err) => Ok(Response::builder()
                    .status(500)
                    .body(Body::from(err.to_string()))
                    .unwrap()),
            }
        })
        .post("/MpiSet", |mut req| async move {
            let body = parse_body::<SetRequest>(&mut req).await.unwrap();
            let platform = req.data::<Arc<Mutex<Platform>>>().unwrap();

            log::info!("MpiSet {:?}", body);

            let component = body.component_name;
            let object = body.object_name;
            let payload = body.payload;
            let payload = serde_json::to_string(&payload).unwrap(); // TODO: this chould be better

            let payload = platform.lock().unwrap().set(&component, &object, &payload);

            match payload {
                Ok(_) => Ok(Response::builder()
                    .status(200)
                    .body(Body::from(""))
                    .unwrap()),
                Err(err) => Ok(Response::builder()
                    .status(500)
                    .body(Body::from(err.to_string()))
                    .unwrap()),
            }
        })
        .post("MpiGetReported", |_| async move {
            // TODO:
            log::info!("MpiGetReported");
            Ok(Response::builder()
            .status(200)
            .body(Body::from("{}"))
            .unwrap())
        })
        .post("/MpiSetDesired", |_| async move {
            // TODO:
            log::info!("MpiSetDesired");
            Ok(Response::builder()
                .status(200)
                .body(Body::from(""))
                .unwrap())
        })
        .post("/MpiOpen", |_| async move {
            // REVIEW: return a session id
            let body = Response::builder()
                .status(200)
                .body(Body::from("\"abc123\""))
                .unwrap();

            log::info!("MpiOpen {:?}", body);
            Ok(body)
        })
        .build()
        .unwrap();

    let service = UnixRouterService::new(router).unwrap();
    Server::bind_unix(path)?.serve(service).await?;

    Ok(())
}

async fn parse_body<T>(req: &mut Request<Body>) -> anyhow::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let body = hyper::body::to_bytes(req.body_mut()).await?;
    let body = String::from_utf8(body.to_vec())?;
    let body = serde_json::from_str(&body)?;
    Ok(body)
}
