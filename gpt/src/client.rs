use anyhow::Result;

use hyper::{Body, Client as SocketClient, Method, Request, Response};
use hyperlocal::{UnixClientExt, UnixConnector, Uri};
use serde_json::Value;

#[derive(Debug)]
pub struct Client {
    socket: String,
    client: SocketClient<UnixConnector>,
}

impl Client {
    pub fn new(socket: &str) -> Self {
        Self {
            socket: socket.to_string(),
            client: SocketClient::unix(),
        }
    }

    pub async fn send(&self, path: &str, method: Method, payload: Option<Value>) -> Result<Response<Body>> {
        let uri = Uri::new(&self.socket, path);
        let body = match payload {
            Some(payload) => Body::from(payload.to_string()),
            None => Body::empty(),
        };
        let req = Request::builder()
            .method(method)
            .uri(uri)
            .body(body)?;
        Ok(self.client.request(req).await?)
    }
}
