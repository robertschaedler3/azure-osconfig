// Copyright (c) Microsoft Corporation. All rights reserved..
// Licensed under the MIT License.

use osc::osc_module;
use thiserror::Error;

#[derive(Default)]
struct Hostname;

#[osc_module]
impl Hostname {
    #[osc(reported)]
    fn name(&self) -> Result<String, Error> {
        let hostname = std::fs::read_to_string("/etc/hostname")?;
        Ok(hostname.trim().to_string())
    }

    #[osc(desired)]
    fn desired_name(&mut self, name: String) -> Result<(), Error> {
        std::fs::write("/etc/hostname", name)?;
        Ok(())
    }
}

#[derive(Debug, Error)]
enum Error {
    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),
}
