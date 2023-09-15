use serde::de::DeserializeOwned;
use warp::Filter;

use crate::{handlers, Platform};

pub fn api(
    platform: Platform,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    open()
        .or(reported(platform.clone()))
        .or(desired(platform.clone()))
        .or(reported_all(platform.clone()))
        .or(desired_all(platform))
        .or(close())
}

pub fn open() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("MpiOpen")
        .and(warp::post())
        .and(json_body())
        .and_then(handlers::open)
}

pub fn close() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("MpiClose")
        .and(warp::post())
        .and(json_body())
        .and_then(handlers::close)
}

pub fn reported(
    platform: Platform,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("MpiGet")
        .and(warp::post())
        .and(with_platform(platform))
        .and(json_body())
        .and_then(handlers::reported)
}

pub fn desired(
    platform: Platform,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("MpiSet")
        .and(warp::post())
        .and(with_platform(platform))
        .and(json_body())
        .and_then(handlers::desired)
}

pub fn reported_all(
    platform: Platform,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("MpiGetReported")
        .and(warp::post())
        .and(with_platform(platform))
        .and(json_body())
        .and_then(handlers::reported_all)
}

pub fn desired_all(
    platform: Platform,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::path!("MpiSetDesired")
        .and(warp::post())
        .and(with_platform(platform))
        .and(json_body())
        .and_then(handlers::desired_all)
}

fn with_platform(
    platform: Platform,
) -> impl Filter<Extract = (Platform,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || platform.clone())
}

fn json_body<T: Send + DeserializeOwned>() -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone {
    // REVIEW: at some point we may want to restrict the size of the body
    // warp::body::content_length_limit(1024 * 10).and(warp::body::json())
    warp::body::json()
}
