// use serde::{Deserialize, Serialize};

// #[derive(Debug, Serialize, Deserialize)]
// #[serde(tag = "type", rename = "Component")]
// struct Component {
//     name: String,
//     properties: Vec<Property>,
// }

// #[derive(Debug, Serialize, Deserialize)]
// #[serde(tag = "type", rename = "Property")]
// struct Property {
//     name: String,
//     schema: Schema,
// }

// #[derive(Debug, Serialize, Deserialize)]
// #[serde(untagged)]
// enum Schema {
//     Primitive(Primitive),
//     IntegerEnum(Enum<i32>),
//     StringEnum(Enum<String>),
//     Object(Object),
//     Array(Array),
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub enum Primitive {
//     #[serde(rename = "string")]
//     String,

//     #[serde(rename = "integer")]
//     Integer,

//     #[serde(rename = "boolean")]
//     Boolean,
// }

// #[derive(Debug, Serialize, Deserialize)]
// #[serde(tag = "type", rename = "enum")]
// #[serde(rename_all = "camelCase")]
// pub struct Enum<T> {
//     pub value_schema: EnumSchema,
//     pub enum_values: Vec<EnumValue<T>>,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub enum EnumSchema {
//     #[serde(rename = "integer")]
//     Integer,

//     #[serde(rename = "string")]
//     String,
// }

// #[derive(Debug, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct EnumValue<T> {
//     pub name: String,
//     pub enum_value: T,
// }

// #[derive(Debug, Serialize, Deserialize)]
// #[serde(tag = "type", rename = "array")]
// #[serde(rename_all = "camelCase")]
// pub struct Array {
//     element_schema: ArraySchema,
// }

// #[derive(Debug, Serialize, Deserialize)]
// #[serde(untagged)]
// pub enum ArraySchema {
//     Primitive(Primitive),
//     Object(Object),
// }

// #[derive(Debug, Serialize, Deserialize)]
// #[serde(tag = "type", rename = "object")]
// pub struct Object {
//     pub fields: Vec<Field<FieldType>>,
// }

// #[derive(Debug, Serialize, Deserialize)]
// #[serde(untagged)]
// pub enum FieldType {
//     Primitive(Primitive),
//     IntegerEnum(Enum<i32>),
//     StringEnum(Enum<String>),
//     Array(Array),
//     // TODO: Map(MapField),
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct Field<T> {
//     pub name: String,
//     pub schema: T,
// }