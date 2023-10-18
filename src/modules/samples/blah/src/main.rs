use blah::{module::Client, *};

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let path = std::env::args().nth(1).expect("Expected a path to a file");
    let client = module::DefaultClient::load(&path)?;

    let meta = client.meta()?;
    log::info!("{:?}", meta);

    let value = client.get("Sample", "simple")?;
    let payload = serde_json::to_string(&value)?;
    log::info!("simple: {}", payload);

    // let value: serde_json::Value = client.get("Sample", "complex_1")?;
    // let payload = serde_json::to_string(&value)?;
    // log::info!("complex_1: {}", payload);

    // let value = client.get("Sample", "complex_2")?;
    // let payload = serde_json::to_string(&value)?;
    // log::info!("complex_2: {}", payload);

    // Set simple
    // let value = serde_json::to_value(42)?;
    // client.set("Sample", "desired_simple", &value)?;

    // let value = client.get("Sample", "simple")?;
    // let payload = serde_json::to_string(&value)?;
    // log::info!("simple: {}", payload);

    let value = serde_json::json!({
        "x": 42,
        "y": "hello"
    });
    client.set("Sample", "desired_complex", &value)?;

    Ok(())
}

struct Foo {
    bar: i32
}


impl Foo {
    fn get_bar(&self) -> i32 {
        self.bar
    }
}