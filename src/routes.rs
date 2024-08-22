use serde::de::DeserializeOwned;
use warp::Filter;

use crate::{handlers, Platform};

pub fn api(
    platform: Platform,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    info(platform.clone())
        .or(reported(platform.clone()))
        .or(desired(platform.clone()))
}

fn info(
    platform: Platform,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("info")
        .and(warp::get())
        .and(with_platform(platform))
        .and_then(handlers::info)
}

fn reported(
    platform: Platform,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!(String / String)
        .and(warp::get())
        .and(with_platform(platform))
        .and_then(
            |component: String, object: String, platform: Platform| async move {
                handlers::reported(platform, &component, &object).await
            },
        )
}

fn desired(
    platform: Platform,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!(String / String)
        .and(warp::post())
        .and(with_platform(platform))
        .and(json_body())
        .and_then(
            |component: String, object: String, platform: Platform, payload| async move {
                handlers::desired(&component, &object, platform, payload).await
            },
        )
}

// fn reported_all(
//     platform: Platform,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     // TODO
// }

// fn desired_all(
//     platform: Platform,
// ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
//     // TODO
// }

fn with_platform(
    platform: Platform,
) -> impl Filter<Extract = (Platform,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || platform.clone())
}

fn json_body<T: Send + DeserializeOwned>(
) -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone {
    // NOTE: at some point we may want to restrict the size of the body
    // warp::body::content_length_limit(1024 * 10).and(warp::body::json())
    warp::body::json()
}
