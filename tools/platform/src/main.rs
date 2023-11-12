use anyhow::{Context, Result};
// use tokio::{
//     net::UnixListener,
//     signal::unix::{signal, SignalKind},
// };
// use tokio_stream::wrappers::UnixListenerStream;
// use warp::Filter;

// use platform::{handlers, routes};

// #[tokio::main]
// async fn main() -> Result<()> {
fn main() -> Result<()> {
    // pretty_env_logger::init();

    // let path = std::path::Path::new("/run/osconfig/mpid.sock");

    // if path.exists() {
    //     std::fs::remove_file(path)?;
    // } else {
    //     let parent = path.parent().context("Unable to get parent directory")?;
    //     std::fs::create_dir_all(parent)?;
    // }

    // let listener = UnixListener::bind(path).context("Unable to bind to unix socket")?;
    // let incoming = UnixListenerStream::new(listener);

    // let mut sigint = signal(SignalKind::interrupt())?;
    // let mut sigquit = signal(SignalKind::quit())?;
    // let mut sigterm = signal(SignalKind::terminate())?;
    // let mut sighup = signal(SignalKind::hangup())?;

    // // REVIEW: initialize the platform asynchronously (server needs to be available ASAP)
    // let platform = platform::init()?;

    // {
    //     let platform = platform.clone();

    //     tokio::spawn(async move {
    //         while let Some(_) = sighup.recv().await {
    //             log::debug!("Received SIGHUP, reloading platform");

    //             let mut platform = platform.lock().unwrap();

    //             if let Err(e) = platform.reload() {
    //                 log::error!("Failed to reload platform: {}", e);
    //             }
    //         }
    //     });
    // }

    // let routes = routes::api(platform)
    //     .with(warp::log("platform"))
    //     .recover(handlers::handle_rejection);

    // let server = warp::serve(routes).serve_incoming_with_graceful_shutdown(incoming, async move {
    //     tokio::select! {
    //         _ = sigint.recv() => {
    //             log::debug!("Received SIGINT, shutting down");
    //         }
    //         _ = sigquit.recv() => {
    //             log::debug!("Received SIGQUIT, shutting down");
    //         }
    //         _ = sigterm.recv() => {
    //             log::debug!("Received SIGTERM, shutting down");
    //         }
    //     }
    // });

    // log::info!("Listening on: {}", path.display());
    // server.await;

    Ok(())
}
