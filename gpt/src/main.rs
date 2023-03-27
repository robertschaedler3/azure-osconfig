use std::{
    env,
    io::{stdin, stdout, Read, Write},
};

use client::Client;
use colored_json::ToColoredJson;
use dotenvy::dotenv;
use hyper::Method;
use openai::{
    chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole},
    set_key,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

mod client;

const PROMPT: &str = r#"You are a helpful assistant that generates JSON payloads to query or change information about a linux device. All of your responses should be in the following form:

```
{ "method": "<GET | POST>", "path": "/<component_name>/<property_name>", "payload": <payload> }
```

where "payload" is only present for POST requests. Given the following schema describing your capabilites, you will be asked to generate the proper payload to achieve the desired outcome. Each response should contain only JSON and no additional text."#;

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    method: String,
    path: String,
    payload: Option<Value>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut file = std::fs::File::open("./schema.json")?;
    let mut schema = String::new();

    let client = Client::new("/run/osconfig/mpid.sock");

    file.read_to_string(&mut schema)?;

    // let components: Vec<Component> = serde_json::from_str(&contents)?;

    // Make sure you have a file named `.env` with the `OPENAI_KEY` environment variable defined!
    dotenv().unwrap();
    set_key(env::var("OPENAI_KEY").unwrap());

    let mut messages = vec![ChatCompletionMessage {
        role: ChatCompletionMessageRole::System,
        content: format!("{}\n\n```\n{}\n```\n", PROMPT, schema),
        name: None,
    }];

    // Open a file to log the conversation (create the file if it doesn't exist)
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("conversation.log")
        .unwrap();

    loop {
        print!("> ");
        stdout().flush().unwrap();

        let mut user_message_content = String::new();

        stdin().read_line(&mut user_message_content).unwrap();

        // Append the user's message to the log file
        writeln!(file, "{}", user_message_content)?;

        messages.push(ChatCompletionMessage {
            role: ChatCompletionMessageRole::User,
            content: user_message_content,
            name: None,
        });

        let chat_completion = ChatCompletion::builder("gpt-3.5-turbo", messages.clone())
            .create()
            .await
            .unwrap()
            .unwrap();
        let returned_message = chat_completion.choices.first().unwrap().message.clone();

        // Append the returned message to the log file
        writeln!(file, "{}", returned_message.content)?;

        let json = extract_json(&returned_message.content)?;
        let message: Message = serde_json::from_value(json)?;
        let method = Method::from_bytes(message.method.as_bytes())?;

        let res = client
            .send(&message.path, method.clone(), message.payload)
            .await?;

        if method == Method::GET {
            let body = hyper::body::to_bytes(res.into_body()).await?;
            let body = String::from_utf8(body.to_vec())?;

            println!("{:?}", body.to_colored_json_auto()?);
        }

        messages.push(returned_message);
    }
}

fn extract_json(message: &str) -> anyhow::Result<Value> {
    if let Some(start) = message.find("```") {
        if let Some(end) = message[start + 3..].find("```") {
            let json = &message[start + 3..start + 3 + end];
            let json = json.trim();

            Ok(serde_json::from_str(json)?)
        } else {
            Ok(serde_json::from_str(message)?)
        }
    } else {
        Ok(serde_json::from_str(message)?)
    }
}


// fn convert() {
//     let mut components = Vec::new();

//     for entry in std::fs::read_dir("gpt/mim")? {
//         let entry = entry?;
//         let path = entry.path();
//         let model = Model::from_file(path)?;

//         for component in model.contents {
//             let mut properties = Vec::new();

//             for object in component.contents {
//                 let name = object.name;
//                 let desired = object.desired;
//                 let schema = match object.schema {
//                     TypeSchema::Primitive(primitive) => Schema::Primitive(match primitive {
//                         Primitive::String => Primitive::String,
//                         Primitive::Integer => Primitive::Integer,
//                         Primitive::Boolean => Primitive::Boolean,
//                     }),
//                     TypeSchema::StringEnum(enum_schema) => {
//                         let value_schema = match enum_schema {
//                             EnumSchema::
//                         }
//                         let enum_value = enum_schema.
//                     }
//                     TypeSchema::IntegerEnum(enum_schema) => {}
//                     TypeSchema::Array(array) => {}
//                     TypeSchema::Object(object) => {}
//                 };
//             }
//         }
//     }
// }
