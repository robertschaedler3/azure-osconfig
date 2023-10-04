use anyhow::{Context, Result};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;
use warp::Filter;

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

    let listener = UnixListener::bind(SOCKET_PATH)?;
    let stream = UnixListenerStream::new(listener);

    // TEMPORARY: Get the modules directory as a commandline argument
    let args: Vec<String> = std::env::args().collect();
    let modules_dir = args.get(1).context("Missing modules directory argument")?;

    let platform = platform::load(modules_dir)?;
    let platform = warp::any().map(move || platform.clone());

    let reported = warp::path!("module" / String / String)
        .and(warp::get())
        .and(platform.clone())
        .map(|module, property, platform| {
            let value = get_reported(module, property, platform).unwrap();
            warp::reply::json(&value)
        });

    let desired = warp::path!("module" / String / String)
        .and(warp::post())
        .and(warp::body::json().map(|body: serde_json::Value| body))
        .and(platform.clone())
        .map(|module, property, body, _| {
            log::debug!("Desired: {:?} {:?} {:?}", module, property, body);
            warp::reply::json(&body)
        });

    let routes = reported.or(desired).with(warp::log("osc-platform"));

    warp::serve(routes).run_incoming(stream).await;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum PlatformResponse {
    #[serde(rename = "data")]
    Success(Value),
    Error(String),
}

// TODO: convert errors into the PlatformResponse::Error variant

fn get_reported(module: String, property: String, platform: Platform) -> Result<PlatformResponse> {
    platform.lock().unwrap().get(&module, &property).map(|value| PlatformResponse::Success(value))
}