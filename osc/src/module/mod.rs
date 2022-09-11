pub mod interface;

use crate::error::Error;

pub trait Module {
    fn new(client_name: &str, max_payload_size: u32) -> Self;

    // fn meta(client_name: &str) -> Meta;

    // TODO: use a better return type (may need Value type for osc_object payloads)
    fn get(&self, component: &str, object: &str) -> Result<Object, Error>;

    // TODO: allow setting complex types (similar to how they are returned from get())
    fn set(&mut self, component: &str, object: &str, value: &str);
}

pub trait Component {
    ///
    fn reported(&self, object: &str) -> Result<Object, Error>;

    ///
    fn desired(&mut self, object: &str, value: &str);
}

pub type Object = serde_json::Value;

pub struct Meta {
    pub name: String,
    // description: String,
    // manufacturer: String,
    // version: String, // TODO: Version struct ???
    pub components: Vec<String>, // TODO: Component struct ???
    // lifetime: i32, // TODO: lifetime enum
    // user_account: i32
}