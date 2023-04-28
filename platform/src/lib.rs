use std::collections::HashMap;

use serde::{Serialize, Deserialize};

pub mod bindings;
pub mod module;
pub mod platform;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Bool(bool),
    Number(i32),
    String(String),
    Array(Vec<ArrayValue>),
    Object(HashMap<String, ObjectValue>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArrayValue {
    Bool(bool),
    Number(i32),
    String(String),
    Object(ObjectValue),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ObjectValue {
    Bool(bool),
    Number(i32),
    String(String),
}
