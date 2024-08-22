use std::convert::Infallible;

use warp::{http::StatusCode, reject::Reject, Rejection, Reply};

use crate::{error::Error, module::Payload, Platform};

impl Reject for crate::error::Error {}

pub async fn info(platform: Platform) -> Result<impl Reply, Rejection> {
    let _platform = platform.lock().unwrap();

    // TODO: get all the info from the modules

    Ok(warp::reply::json(&123))
}

pub async fn reported(platform: Platform, component: &str, object: &str) -> Result<impl Reply, Rejection> {
    let platform = platform.lock().unwrap();
    let value = platform.get(component, object)?;
    Ok(warp::reply::json(&value))
}

pub async fn desired(component: &str, object: &str, platform: Platform, payload: Payload) -> Result<impl Reply, Rejection> {
    let platform = platform.lock().unwrap();
    let value = platform.set(&component, &object, &payload)?;
    Ok(warp::reply::json(&value))
}

pub async fn reported_all(
    platform: Platform,
) -> Result<impl Reply, Infallible> {
    let platform = platform.lock().unwrap();

    let _reported = &platform.config.reported.0;

    // Iterate over reported component/object pairs and build up a JSON object by calling get() on each pair.

    let all: Vec<String> = Vec::new();
    // let all = reported
    //     .iter()
    //     .fold(serde_json::json!({}), |mut acc, (component, objects)| {
    //         let mut component_json = serde_json::json!({});
    //         for object in objects {
    //             let value = platform.get(&component, &object);
    //             match value {
    //                 Ok(value) => {
    //                     log::debug!("{}.{}: {}", component, object, value);
    //                     component_json[object] = value;
    //                 }
    //                 Err(e) => {
    //                     // These errors are usually a bug in the module or a typo in osconfig.json
    //                     log::warn!("{}: {}", object, e);
    //                 }
    //             }
    //         }
    //         acc[component] = component_json;
    //         acc
    //     });

    Ok(warp::reply::json(&all))
}

pub async fn desired_all(
    _platform: Platform,
    _payload: Payload
) -> Result<impl Reply, Rejection> {
    // let platform = platform.lock().unwrap();

    // Iterate over the JSON object and call set() on each component/object pair.

    // TODO: handle unwrap()
    // for (component, objects) in payload.as_object().unwrap() {
    //     for (object, payload) in objects.as_object().unwrap() {
    //         if let Err(e) = platform.set(&component, &object, &payload) {
    //             log::error!("{}.{}: {}", component, object, e);
    //         }
    //     }
    // }

    Ok(warp::reply())
}

/// Handles a rejection by logging the error and returning a JSON response with the error message.
/// If the error is a `crate::error::Error`, the error message will be returned. Otherwise, a
/// generic "Internal Server Error" message will be returned.
pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let (code, message) = match err.find::<crate::error::Error>() {
        Some(err) => {
            log::error!("{}", err);
            match err {
                Error::ComponentNotFound(component) => {
                    (StatusCode::NOT_FOUND, format!("Component not found: {}", component))
                }
                Error::Json(err) => (StatusCode::BAD_REQUEST, err.to_string()),
                Error::Null(err) => (StatusCode::BAD_REQUEST, err.to_string()),
                Error::Io(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
                Error::Library(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
                Error::Errno(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.0.to_string()),
                // _ => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            }
        }
        None => {
            log::error!("Unhandled rejection: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error".to_string())
        }
    };

    let json = warp::reply::json(&serde_json::json!( {
        "error": message,
    }));

    Ok(warp::reply::with_status(json, code))
}
