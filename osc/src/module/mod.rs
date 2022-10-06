pub mod interface;

use crate::error::Error;

// TODO: use custom MIM types
pub type Object = serde_json::Value;

pub struct Module<ComponentT: Component> {
    _client_name: String,
    _max_payload_size: u32,
    component: ComponentT,
}

pub trait Interface {
    ///
    fn open(client_name: &str, max_payload_size: u32) -> Self;

    // TODO: meta() for get_info

    ///
    fn get(&self, component_name: &str, object_name: &str) -> Result<Object, Error>;

    ///
    fn set(&mut self, component_name: &str, object_name: &str, value: &str) -> Result<(), Error>;
}

impl<ComponentT> Interface for Module<ComponentT>
where
    ComponentT: Component + Default,
{
    fn open(client_name: &str, max_payload_size: u32) -> Self {
        Self {
            _client_name: client_name.to_string(),
            _max_payload_size: max_payload_size,
            component: ComponentT::default(),
        }
    }

    fn get(&self, component_name: &str, object_name: &str) -> Result<Object, Error> {
        if component_name == self.component.name() {
            self.component.reported(object_name)
        } else {
            Err(Error::from(format!("unknown component: {}", component_name)))
        }
    }

    fn set(&mut self, component_name: &str, object_name: &str, value: &str) -> Result<(), Error> {
        if component_name == self.component.name() {
            let value = serde_json::from_str::<Object>(value)?;
            self.component.desired(object_name, value)
        } else {
            Err(Error::from(format!("unknown component: {}", component_name)))
        }
    }
}

pub trait Component {
    /// The name of the component
    fn name(&self) -> &str;

    // fn meta(&self) -> Meta;

    /// Gets a reported object
    fn reported(&self, object_name: &str) -> Result<Object, Error>;

    /// Sets a desired object with the given value
    fn desired(&mut self, object_name: &str, value: Object) -> Result<(), Error>;
    // TODO: fn desired<T>(&mut self, object_name: &str, value: Object) -> Result<T, Error>;
}

// TODO: Meta struct for "GetInfo()" type
// TODO: Ideally this struct should be able to contain a full schema of the component
//       and its properties/objects (especaiily easy to add with macros)

// pub struct Meta {
//     pub name: String,
//     // description: String,
//     // manufacturer: String,
//     // version: String, // TODO: Version struct ???
//     pub components: Vec<String>, // TODO: Component struct ???
//     // lifetime: i32, // TODO: lifetime enum
//     // user_account: i32
// }