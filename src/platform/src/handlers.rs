use std::convert::Infallible;

use warp::{http::StatusCode, reject::Reject, Rejection, Reply};

use crate::{
    config::Reported,
    error::Error,
    models::{CloseBody, GetBody, GetReportedBody, OpenBody, SetBody, SetDesiredBody}, Data,
};

impl Reject for crate::error::Error {}

pub async fn open(body: OpenBody) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&body.client_name))
}

pub async fn close(body: CloseBody) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&body.client_session))
}

pub async fn reported(
    data: Data,
    GetBody {
        component, object, ..
    }: GetBody,
) -> Result<impl Reply, Rejection> {
    let platform = data.lock().unwrap();
    Ok(warp::reply::json(&platform.get(&component, &object)?))
}

pub async fn desired(
    data: Data,
    SetBody {
        component,
        object,
        payload,
        ..
    }: SetBody,
) -> Result<impl Reply, Rejection> {
    let platform = data.lock().unwrap();
    platform.set(&component, &object, &payload)?;
    Ok(warp::reply())
}

pub async fn reported_all(data: Data, _body: GetReportedBody) -> Result<impl Reply, Infallible> {
    let platform = data.lock().unwrap();

    let Reported(reported) = &platform.config.reported;

    // Iterate over reported component/object pairs and build up a JSON object by calling get() on each pair.

    let all = reported
        .iter()
        .fold(serde_json::json!({}), |mut acc, (component, objects)| {
            let mut component_json = serde_json::json!({});
            for object in objects {
                let value = platform.get(&component, &object);
                match value {
                    Ok(value) => {
                        log::debug!("{}.{}: {}", component, object, value);
                        component_json[object] = value;
                    }
                    Err(e) => {
                        // These errors are usually a bug in the module or a typo in osconfig.json
                        log::warn!("{}: {}", object, e);
                    }
                }
            }
            acc[component] = component_json;
            acc
        });

    Ok(warp::reply::json(&all))
}

pub async fn desired_all(data: Data, body: SetDesiredBody) -> Result<impl Reply, Rejection> {
    let platform = data.lock().unwrap();

    // Iterate over the JSON object and call set() on each component/object pair.

    for (component, objects) in body.payload {
        for (object, payload) in objects {
            if let Err(e) = platform.set(&component, &object, &payload) {
                log::error!("{}.{}: {}", component, object, e);
            }
        }
    }

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
                Error::ComponentNotFound(component) => (
                    StatusCode::NOT_FOUND,
                    format!("Component not found: {}", component),
                ),
                Error::Json(err) => (StatusCode::BAD_REQUEST, err.to_string()),
                Error::Null(err) => (StatusCode::BAD_REQUEST, err.to_string()),
                Error::Io(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
                Error::Library(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
                Error::Errno(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.0.to_string()),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            }
        }
        None => {
            log::error!("Unhandled rejection: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error".to_string(),
            )
        }
    };

    let json = warp::reply::json(&serde_json::json!( {
        "error": message,
    }));

    Ok(warp::reply::with_status(json, code))
}
