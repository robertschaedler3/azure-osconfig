use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, fs};

#[derive(Debug)]
pub struct Schema {
    schema: HashMap<String, HashMap<String, MimObjectSchema>>,
    errors: Vec<Box<dyn Error>>,
}

// TODO: custom shema error type

impl Schema {
    pub fn load(dir: String) -> Result<Self, Box<dyn Error>> {
        let mut schema = HashMap::new();
        let mut errors = Vec::new();
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let path_str = path.to_str().unwrap();
            let json = fs::read_to_string(path_str)?;
            let mim = serde_json::from_str::<MimSchema>(&json);
            match mim {
                Ok(mim) => {
                    for component in mim.contents.iter() {
                        // TODO: check for duplicate components
                        schema.insert(
                            component.name.clone(),
                            component
                                .contents
                                .iter()
                                .map(|MimObject::MimObject(object)| {
                                    (object.name.clone(), object.clone())
                                })
                                .collect(),
                        );
                    }
                }
                Err(e) => {
                    errors.push(format!("{}: {}", path_str, e).into());
                }
            }
        }
        Ok(Self { schema, errors })
    }

    pub fn errors(&self) -> &[Box<dyn Error>] {
        &self.errors
    }

    pub fn components(&self) -> Vec<String> {
        self.schema.keys().map(|k| k.to_string()).collect()
    }

    pub fn objects(&self, component: &str) -> Vec<String> {
        self.schema
            .get(component)
            .unwrap()
            .keys()
            .map(|k| k.to_string())
            .collect()
    }

    /// Returns a list of components that have objects with reported settings
    pub fn reported_components(&self) -> Vec<String> {
        self.schema
            .iter()
            .filter_map(|(component, objects)| {
                if objects.iter().any(|(_, object)| !object.desired) {
                    Some(component.to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns a list of components that have objects with desired settings
    pub fn desired_components(&self) -> Vec<String> {
        self.schema
            .iter()
            .filter_map(|(component, objects)| {
                if objects.iter().any(|(_, object)| object.desired) {
                    Some(component.to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    // Returns a list of objects with reported settings for a given component
    pub fn reported_objects(&self, component: &String) -> Vec<String> {
        self.schema
            .get(component)
            .unwrap()
            .iter()
            .filter_map(|(object, object_schema)| {
                if !object_schema.desired {
                    Some(object.to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns a list of objects with desired settings for a given component
    pub fn desired_objects(&self, component: &String) -> Vec<String> {
        self.schema
            .get(component)
            .unwrap()
            .iter()
            .filter_map(|(object, object_schema)| {
                if object_schema.desired {
                    Some(object.to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns the schema for a given component and object
    pub fn setting(&self, component: &String, object: &String) -> TypeSchema {
        // TODO: handle errors
        self.schema
            .get(component)
            .unwrap()
            .get(object)
            .unwrap()
            .schema
            .clone()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct MimSchema {
    name: String,
    // type: String
    contents: Vec<MimComponentSchema>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MimComponentSchema {
    name: String,
    // type: String,
    contents: Vec<MimObject>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum MimObject {
    #[serde(rename = "mimObject")]
    MimObject(MimObjectSchema),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct MimObjectSchema {
    name: String,
    desired: bool,
    schema: TypeSchema,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TypeSchema {
    Primitive(Primitive),
    Enum(Enum),
    Array(Array),
    Object(Object),
    // Map(Map),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Primitive {
    #[serde(rename = "string")]
    String,

    #[serde(rename = "integer")]
    Integer,

    #[serde(rename = "boolean")]
    Boolean,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename = "enum")]
pub struct Enum {
    #[serde(rename = "valueSchema")]
    value_schema: EnumSchema,

    #[serde(rename = "enumValues")]
    values: Vec<EnumValue>,
}

impl Enum {
    pub fn values(&self) -> Vec<EnumValue> {
        self.values.clone()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EnumSchema {
    #[serde(rename = "integer")]
    Integer,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnumValue {
    name: String,

    #[serde(rename = "enumValue")]
    enum_value: u32,
}

impl EnumValue {
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename = "array")]
pub struct Array {
    #[serde(rename = "elementSchema")]
    element_schema: ArraySchema,
}

impl Array {
    pub fn schema(&self) -> TypeSchema {
        match &self.element_schema {
            ArraySchema::Primitive(primitive) => match primitive {
                ArrayPrimitive::String => TypeSchema::Primitive(Primitive::String),
                ArrayPrimitive::Integer => TypeSchema::Primitive(Primitive::Integer),
            },
            ArraySchema::Object(object_schema) => TypeSchema::Object(object_schema.clone()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArraySchema {
    Primitive(ArrayPrimitive),
    Object(Object),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ArrayPrimitive {
    #[serde(rename = "string")]
    String,

    #[serde(rename = "integer")]
    Integer,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename = "object")]
pub struct Object {
    fields: Vec<Field>,
}

impl Object {
    pub fn fields(&self) -> Vec<Field> {
        self.fields.clone()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Field {
    Primitive(PrimitiveField),
    Enum(EnumField),
    Array(ArrayField),
    // Map(MapField),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrimitiveField {
    name: String,
    schema: Primitive,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnumField {
    name: String,
    schema: Enum,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArrayField {
    name: String,
    schema: ArrayFieldSchema,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ArrayFieldSchema {
    #[serde(rename = "string")]
    String,

    #[serde(rename = "integer")]
    Integer,
}

impl Field {
    pub fn name(&self) -> String {
        match self {
            Field::Primitive(primitive) => primitive.name.clone(),
            Field::Enum(enum_field) => enum_field.name.clone(),
            Field::Array(array_field) => array_field.name.clone(),
            // Field::Map(map_field) => map_field.name.clone(),
        }
    }

    pub fn schema(&self) -> TypeSchema {
        match self {
            Field::Primitive(primitive) => primitive.schema.clone().into(),
            Field::Enum(enum_field) => enum_field.schema.clone().into(),
            Field::Array(array_field) => array_field.schema.clone().into(),
            // Field::Map(map_field) => map_field.schema.clone(),
        }
    }
}

impl From<Primitive> for TypeSchema {
    fn from(primitive: Primitive) -> Self {
        TypeSchema::Primitive(primitive)
    }
}

impl From<Enum> for TypeSchema {
    fn from(enum_: Enum) -> Self {
        TypeSchema::Enum(enum_)
    }
}

impl From<ArrayFieldSchema> for TypeSchema {
    fn from(array: ArrayFieldSchema) -> Self {
        match array {
            ArrayFieldSchema::String => TypeSchema::Primitive(Primitive::String),
            ArrayFieldSchema::Integer => TypeSchema::Primitive(Primitive::Integer),
        }
    }
}
