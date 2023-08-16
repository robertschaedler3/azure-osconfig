// Controller responsible for handling all of the module clients and their sessions
// The controller is responsible for loading the modules in separate processes and managing their sessions

trait Controller {
    type Context;

    fn init(&self, context: Self::Context) -> Result<()>;

    fn get(&self, component: &str, object: &str) -> Result<Value>;

    fn set(&self, component: &str, object: &str, value: &Value) -> Result<()>;
}

// struct Orchestrator {

// }