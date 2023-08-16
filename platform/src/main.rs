use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
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

    let platform = Arc::new(Mutex::new(Platform::new(&modules_dir)?));
    let platform = warp::any().map(move || platform.clone());

    // NOTE: placeholder routes

    let read = warp::path!("read")
        .and(warp::get())
        .and(platform.clone())
        .map(|platform| "TODO: implement read");

    let write = warp::path!("write")
        .and(warp::post())
        .and(warp::body::json().map(|body: serde_json::Value| body))
        .and(platform.clone())
        .map(|body, platform| "TODO: implement write");

    let routes = read.or(write).with(warp::log("osc-platform"));

    warp::serve(routes).run_incoming(stream).await;

    Ok(())
}
