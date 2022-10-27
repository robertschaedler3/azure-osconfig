use format_serde_error::SerdeError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
    pub name: String,
    // TODO: type: "mimModel",
    pub contents: Vec<Component>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    // TODO: type: "mimComponent",
    pub contents: Vec<Object>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Object {
    pub name: String,
    pub desired: bool,
    pub schema: TypeSchema,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TypeSchema {
    Primitive(PrimitiveType),

    IntegerEnum(EnumType<i32>),
    StringEnum(EnumType<String>),

    Array(ArrayType),
    Object(ObjectType),
    // TODO: Map(MapType),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PrimitiveType {
    #[serde(rename = "string")]
    String,

    #[serde(rename = "integer")]
    Integer,

    #[serde(rename = "boolean")]
    Boolean,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename = "enum")]
#[serde(rename_all = "camelCase")]
pub struct EnumType<T> {
    pub value_schema: EnumSchema,
    pub enum_values: Vec<EnumValue<T>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EnumSchema {
    #[serde(rename = "integer")]
    Integer,

    #[serde(rename = "string")]
    String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnumValue<T> {
    pub name: String,
    pub enum_value: T,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename = "array")]
#[serde(rename_all = "camelCase")]
pub struct ArrayType {
    element_schema: ArraySchema,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArraySchema {
    Primitive(PrimitiveType),
    Object(ObjectType),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename = "object")]
pub struct ObjectType {
    pub fields: Vec<Field<FieldType>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FieldType {
    Primitive(PrimitiveType),
    // TODO: enums (integer, string)
    Enum(EnumType<i32>),
    EnumString(EnumType<String>),
    Array(ArrayType),
    // TODO: Map(MapField),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Field<T> {
    pub name: String,
    pub schema: T,
}

impl Model {
    pub fn from_file(path: PathBuf) -> anyhow::Result<Model> {
        let s = std::fs::read_to_string(path)?;
        let model: Model =
            serde_json::from_str(s.as_str()).map_err(|err| SerdeError::new(s.to_string(), err))?;
        Ok(model)
    }
}
