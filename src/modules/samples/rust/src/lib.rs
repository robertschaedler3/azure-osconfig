// Copyright (c) Microsoft Corporation. All rights reserved..
// Licensed under the MIT License.

use thiserror::Error;

use osc::osc_module;

#[derive(Default)]
struct Hostname;

#[osc_module(
    name = "Hostname",
    description = "Provides functionality to observe and configure network hostname and hosts",
    manufacturer = "Microsoft",
    version = "1.0"
)]
impl Hostname {
    #[osc(reported)]
    fn name() -> Result<String, HostnameError> {
        Ok(hostname::get()?.to_string_lossy().into())
    }

    #[osc(desired)]
    fn desired_name(name: String) -> Result<(), HostnameError> {
        hostname::set(&name)?;
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum HostnameError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
