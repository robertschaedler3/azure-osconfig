// Copyright (c) Microsoft Corporation. All rights reserved..
// Licensed under the MIT License.

use osc::osc_module;
use serde::{Serialize, Deserialize};

#[derive(Default)]
struct Sample {
    x: i32,
}

#[osc_module(
    // name = "Blah",
    description = "Sample module for...",
    manufacturer = "Microsoft",
    version = "1.0",
    lifetime = "long",
    // user_account = 0
)]
impl Sample {
    #[osc(desired)]
    fn desired_simple(&mut self, x: i32) {
        // fn desired_simple(x: i32) {
            self.x = x;
        println!("desired_simple: {}", x)
    }

    #[osc(reported)]
    fn simple(&self) -> Option<i32> {
        // REVIEW: this result type doesnt work
        // fn simple() -> Result<i32, CustomError> { // REVIEW: this result type doesnt work
        // fn simple() -> i32 { // REVIEW: this result type doesnt work
        // Ok(42)
        Some(self.x)
        // None
    }

    #[osc(reported)]
    fn complex_1() -> Foo {
        Foo {
            x: 42,
            y: "hello".to_string(),
        }
    }

    #[osc(write)]
    fn desired_complex(foo: Foo) {
        println!("desired_complex: {:?}", foo);
    }

    // #[osc(reported)]
    // fn complex_2() -> Result<Vec<Foo>, CustomError> {
    //     // Ok(vec![
    //     //     Foo {
    //     //         x: 42,
    //     //         y: "hello".to_string(),
    //     //     },
    //     //     Foo {
    //     //         x: 43,
    //     //         y: "world".to_string(),
    //     //     },
    //     // ])
    //     Err(CustomError::Something)
    // }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct Foo {
    x: i32,
    y: String,
}

#[derive(thiserror::Error, Debug)]
enum CustomError {
    #[error("Something went wrong")]
    Something,
}
