// SPDX-FileCopyrightText: 2021 HH Partners
//
// SPDX-License-Identifier: MIT

//! # Fossology
//!
//! Module for communicating with Fossology's REST API.

#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(
    clippy::must_use_candidate,
    clippy::module_name_repetitions,
    clippy::option_if_let_else,
    clippy::struct_excessive_bools
)]

use log::error;
use reqwest::blocking::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};

pub mod auth;
pub mod info;
pub mod job;
pub mod license;
pub mod upload;
mod utilities;

/// Fossology instance.
#[derive(Debug)]
pub struct Fossology {
    /// API base uri.
    uri: String,

    /// Access token for Fossology.
    token: String,

    /// Reqwest client.
    client: Client,
}

/// Error when interacting with Fossology.
#[derive(Debug, thiserror::Error)]
pub enum FossologyError {
    #[error(transparent)]
    FileError(#[from] std::io::Error),

    #[error(transparent)]
    RequestError(#[from] reqwest::Error),

    #[error("Unexpected response: {0}")]
    UnexpectedResponse(String),

    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),

    #[error("Error: {0}")]
    Other(String),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum FossologyResponse<T> {
    Response(T),
    ApiError(Info),
}

#[derive(Debug, Deserialize)]
pub struct Info {
    pub code: i32,
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
}

#[derive(Debug, Deserialize)]
pub struct InfoWithNumber {
    pub code: i32,
    pub message: i32,
    #[serde(rename = "type")]
    pub error_type: String,
}

// TODO: Can be deleted.
/// Objects in downloads-folder to be uploaded to Fossology.
#[derive(Debug, Serialize, Deserialize)]
pub struct UploadObject {
    path: String,
    sha256: String,
    exists_in_fossology: bool,
}

impl Fossology {
    /// Initialize Fossology with URI and token.
    pub fn new(uri: &str, token: &str) -> Self {
        Self {
            uri: uri.to_owned(),
            token: token.to_owned(),
            client: Client::new(),
        }
    }

    pub(crate) fn init_get_with_token(&self, path: &str) -> RequestBuilder {
        self.client
            .get(&format!("{}/{}", self.uri, path))
            .bearer_auth(&self.token)
    }

    pub(crate) fn init_post_with_token(&self, path: &str) -> RequestBuilder {
        self.client
            .post(&format!("{}/{}", self.uri, path))
            .bearer_auth(&self.token)
    }
}

#[cfg(test)]
mod tests {
    use super::Fossology;
    use reqwest::blocking::Client;

    #[test]
    fn fossology_is_created() {
        let expected_fossology = Fossology {
            token: "token".into(),
            uri: "uri".into(),
            client: Client::new(),
        };

        let fossology = Fossology::new("uri", "token");

        assert_eq!(fossology.token, expected_fossology.token);
        assert_eq!(fossology.uri, expected_fossology.uri);
    }
}
