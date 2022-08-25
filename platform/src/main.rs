use hyper::{Body, Response, Server, Request};
use hyperlocal::UnixServerExt;
use routerify::{Error, Router};
use routerify_unixsocket::{UnixRequestExt, UnixRouterService};
use std::{fs, path::Path};
// use serde::{de, Deserialize};

// mod mmi;
// use mmi::ModuleManager;

// #[derive(Debug, Deserialize)]
// struct MmiGet {
//     client_name: String,
//     component: String,
//     object: String
// }

// async fn deserialize<T>(req: &mut Request<Body>) -> serde_json::Result<T>
//     where for<'de> T: de::Deserialize<'de>,
// {
//     let body = hyper::body::to_bytes(req.body_mut()).await.unwrap();
//     println!("{:?}", body);
//     // let (parts, body) = req.into_parts();
//     serde_json::from_slice(&body)
// }

async fn log_body(req: &mut Request<Body>) {
    let body = hyper::body::to_bytes(req.body_mut()).await.unwrap();
    println!("{:?}", body);
}

#[tokio::main]
async fn main() {
    // Set log level

    let path = Path::new("/run/osconfig/mpid.sock");
    if path.exists() {
        fs::remove_file(path).unwrap();
    }

    // let mut mm: Box<Mutex<ModuleManager>> = Box::new(Mutex::new(ModuleManager::default()));

    // let mut mm = ModuleManager::default();

    let router: Router<Body, Error> = Router::builder()
    .post("/MpiOpen", |mut req| async move {
        println!("mpi open: {:?}", req.unix_peer_cred());
        log_body(&mut req).await;
        Ok(Response::new(Body::from("\"abc123\"")))
    })
    .post("/MpiClose", |mut req| async move {
        println!("mpi close: {:?}", req.unix_peer_cred());
        log_body(&mut req).await;
        Ok(Response::new(Body::from("close request")))
    })
    .post("/MpiGet", |mut req| async move {
        println!("mpiget: {:?}", req.unix_peer_cred());
        log_body(&mut req).await;
        Ok(Response::new(Body::from("get request")))
    })
    .post("/MpiSet", |mut req| async move {
        println!("mpi set: {:?}", req.unix_peer_cred());
        log_body(&mut req).await;
        Ok(Response::new(Body::from("set request")))
    })
    // .get("/mmi/open", |req| async move {
        //     info!("You are: {:?}", req.unix_peer_cred());
        //     Ok(Response::new(Body::from(s)))
        // })
        // .post("/mmi/close", |req| async move {
        //     info!("You are: {:?}", req.unix_peer_cred());
        //     Ok(Response::new(Body::from(s)))
        // })
        // .get("/mmi", |mut req| async move {

        //     let mut mm = ModuleManager::default();
        //     mm.load("Blah", "/usr/lib/osconfig");
        //     println!("MMIGET Request: {:?}", req.unix_peer_cred());

        //     let payload = deserialize::<MmiGet>(&mut req).await.unwrap();
        //     println!("{:?}", payload);
        //     // let body = hyper::body::to_bytes(req.body_mut()).await.unwrap();
        //     // println!("{:?}", body);

        //     let client = payload.client_name.clone();
        //     let component = payload.component.clone();
        //     let object = payload.object;
        //     // let mut mm = mm.lock().unwrap();
        //     let result = mm.get(&client, &component, &object);
        //     match result {
        //         Ok(payload) => {
        //             let body = Body::from(payload);
        //             Ok(Response::new(body))
        //         }
        //         Err(e) => {
        //             let body = Body::from("Error");
        //             let response = Response::builder()
        //                 .status(500)
        //                 .body(body)
        //                 .unwrap();
        //             Ok(response)
        //         }
        //     }

        //     // mm.get(&client, &component, &object); // TODO: this needs to send a message on a channel (where the MM is listening) and wait for a response
        // })
        // .post("/mmi", |req| async move {
        //     println!("MMISET Request: {:?}", req.unix_peer_cred());

        //     Ok(Response::new(Body::from("get request")))
        // })
        .build()
        .unwrap();

    let service = UnixRouterService::new(router).unwrap();
        Server::bind_unix(path)
            .unwrap()
            .serve(service)
            .await
            .unwrap()
}