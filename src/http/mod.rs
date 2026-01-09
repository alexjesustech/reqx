// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! HTTP client module

mod client;

pub use client::Client;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// HTTP response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: serde_json::Value,
    pub duration: Duration,
}

/// HTTP configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HttpConfig {
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default = "default_true")]
    pub follow_redirects: bool,
    #[serde(default = "default_max_redirects")]
    pub max_redirects: usize,
    pub proxy: Option<String>,
    pub ca_cert: Option<String>,
    pub client_cert: Option<String>,
    pub client_key: Option<String>,
    #[serde(default)]
    pub insecure: bool,
}

fn default_timeout() -> u64 {
    30000
}

fn default_true() -> bool {
    true
}

fn default_max_redirects() -> usize {
    10
}
