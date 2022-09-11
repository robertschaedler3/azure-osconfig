use hyper::{Body, Client as SocketClient, Method, Request};
use hyperlocal::{UnixClientExt, UnixConnector, Uri};
use serde::Serialize;
use serde_json::Value;
use std::error::Error;

// TODO: consolidate/flatten these types ?

#[derive(Debug, Serialize)]
struct MpiOpen {
    #[serde(rename = "ClientName")]
    client_name: String,

    #[serde(rename = "MaxPayloadSizeBytes")]
    max_payload_size: u64,
}

#[derive(Debug, Serialize)]
struct MpiClose {
    #[serde(rename = "ClientSession")]
    session: String,
}

#[derive(Debug, Serialize)]
struct MpiGet {
    #[serde(rename = "ClientSession")]
    session: String,

    #[serde(rename = "ComponentName")]
    component: String,

    #[serde(rename = "ObjectName")]
    object: String,
}

#[derive(Debug, Serialize)]
struct MpiGetReported {
    #[serde(rename = "ClientSession")]
    session: String,
}

#[derive(Debug, Serialize)]
struct MpiSet {
    #[serde(rename = "ClientSession")]
    session: String,

    #[serde(rename = "ComponentName")]
    component: String,

    #[serde(rename = "ObjectName")]
    object: String,

    #[serde(rename = "Payload")]
    value: Value,
}

#[derive(Debug)]
pub struct Client {
    client: SocketClient<UnixConnector>,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            client: SocketClient::unix(),
        }
    }
}

// Marker trait for the value equivalent of a MIM type (schema)
// TODO: move this to mim::schema ???
// trait ValueType {}

// TODO: use this for all plaform responses (Result<Response, Error>)
// maybe just use hyper::Response ???

// struct Response<T>
// where
//     T: ValueType,
// {
//     time: u64, // TODO: use chrono::DateTime
//     status: u64, // TODO: use hyper::StatusCode
//     body: T,
// }

impl Client {
    async fn send<T>(&self, path: &str, body: T) -> Result<String, Box<dyn Error + Send + Sync>>
    where
        T: Serialize,
    {
        let uri: Uri = Uri::new("/run/osconfig/mpid.sock", path);
        let payload = serde_json::to_string(&body)?;
        println!("{}", payload);
        let req = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .body(Body::from(payload))?;
        let res = self.client.request(req).await?;
        // Check status code
        if res.status() == 200 {
            let body = hyper::body::to_bytes(res.into_body()).await?;
            let body = String::from_utf8(body.to_vec())?;
            Ok(body)
        } else {
            // TODO: check for payload?
            Err(format!("{}", res.status()).into())
        }
    }

    pub async fn open(
        &self,
        client_name: String,
        max_payload_size: u64,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let body = MpiOpen {
            client_name,
            max_payload_size,
        };
        let res = self.send("/MpiOpen", body).await?;
        let session_id = serde_json::from_str::<String>(&res)?;
        Ok(session_id)
    }

    pub async fn close(&self, session: String) -> Result<(), Box<dyn Error + Send + Sync>> {
        let body = MpiClose { session };
        self.send("/MpiClose", body).await?;
        Ok(())
    }

    // TODO: return an error if request status code is not 200
    pub async fn get(
        &self,
        session: String,
        component: String,
        object: String,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let body = MpiGet {
            session,
            component,
            object,
        };
        self.send("/MpiGet", body).await
    }

    pub async fn get_reported(
        &self,
        session: String,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let body = MpiGetReported { session };
        self.send("/MpiGetReported", body).await
    }

    // TODO: return an error if request status code is not 200
    pub async fn set(
        &self,
        session: String,
        component: String,
        object: String,
        value: Value,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let body = MpiSet {
            session,
            component,
            object,
            value,
        };
        self.send("/MpiSet", body).await
    }
}
