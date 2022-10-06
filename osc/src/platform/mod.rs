use hyper::{Body, Client as SocketClient, Method, Request};
use hyperlocal::{UnixClientExt, UnixConnector, Uri};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::error::Error;

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
pub trait ValueType: DeserializeOwned {}

// TODO: use this for all plaform responses (Result<Response, Error>)
// maybe just use hyper::Response ???

struct Response<T>
where
    // T: ValueType,
    T: DeserializeOwned,
{
    // time: u64, // TODO: use chrono::DateTime
    // status: http::StatusCode, // TODO: use hyper::StatusCode
    payload: T,
}

impl Client {
    // TODO: maybe return generic Result<Response, Error> where Reponse: Deserialize
    //
    // (also need to include the return code and other response metadata)
    // async fn send<T>(&self, path: &str, body: T) -> Result<String, Box<dyn Error + Send + Sync>>
    // where
    // T: Serialize,
    async fn send<Req, Res>(
        &self,
        path: &str,
        req: Req,
    ) -> Result<Response<Res>, Box<dyn Error + Send + Sync>>
    where
        Req: Serialize,
        // Res: ValueType,
        Res: DeserializeOwned,
    {
        let uri: Uri = Uri::new("/run/osconfig/mpid.sock", path);
        let payload = serde_json::to_string(&req)?;

        // TODO: log level verbose only
        println!("Sending payload ({}): {}", path, payload);

        let req = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .body(Body::from(payload))?;
        let res = self.client.request(req).await?;
        // let status = res.status();
        let body = hyper::body::to_bytes(res.into_body()).await?;
        let body = String::from_utf8(body.to_vec())?;
        let payload = serde_json::from_str(&body)?;

        // Ok(Response { status, payload })
        Ok(Response { payload })
    }

    pub async fn open(
        &self,
        client_name: String,
        max_payload_size_bytes: u64,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let body = OpenRequest {
            client_name,
            max_payload_size_bytes,
        };
        Ok(self.send("/MpiOpen", body).await?.payload)
    }

    // pub async fn close(&self, client_session: String) -> Result<Response<()>, Box<dyn Error + Send + Sync>> {
    //     let body = CloseRequest { client_session };
    //     Ok(self.send::<CloseRequest, ()>("/MpiClose", body).await?)
    // }

    // TODO: return an error if request status code is not 200
    pub async fn get<T>(
        &self,
        client_session: String,
        component_name: String,
        object_name: String,
    ) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        T: ValueType,
    {
        let body = GetRequest {
            client_session,
            component_name,
            object_name,
        };
        Ok(self.send("/MpiGet", body).await?.payload)
    }

    pub async fn get_reported(
        &self,
        client_session: String,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let body = GetReportedRequest { client_session };
        Ok(self.send("/MpiGetReported", body).await?.payload)
    }

    pub async fn set_desired(
        &self,
        _session: String,
        _value: Value,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // let body = SetRequest {
        //     session,
        //     component: "desired".to_string(),
        //     object: "state".to_string(),
        //     value,
        // };
        // self.send("/MpiSet", body).await?;
        Ok(())
    }

    // TODO: return an error if request status code is not 200
    pub async fn set(
        &self,
        client_session: String,
        component_name: String,
        object_name: String,
        payload: Value,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let body = SetRequest {
            client_session,
            component_name,
            object_name,
            payload,
        };
        Ok(self.send("/MpiSet", body).await?.payload)
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct OpenRequest {
    client_name: String,
    max_payload_size_bytes: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct CloseRequest {
    client_session: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct GetRequest {
    client_session: String,
    component_name: String,
    object_name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct GetReportedRequest {
    client_session: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct SetRequest {
    client_session: String,
    component_name: String,
    object_name: String,
    payload: Value,
}
