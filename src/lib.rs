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
use serde::Deserialize;
use version_compare::{CompOp, VersionCompare};

use crate::info::{ApiInformation, ApiInformationV1};

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

    version: String,
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

    #[error("Fossology version does not support the endpoint.")]
    UnsupportedVersion,

    #[error("Error: {0}")]
    Other(String),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum FossologyResponse<T> {
    Response(T),
    ApiError(Info),
}

impl<T> FossologyResponse<T> {
    #[allow(clippy::missing_const_for_fn)]
    pub(crate) fn return_response_or_error(self) -> Result<T, FossologyError> {
        match self {
            FossologyResponse::Response(res) => Ok(res),
            FossologyResponse::ApiError(err) => {
                Err(FossologyError::UnexpectedResponse(err.message))
            }
        }
    }
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

impl Fossology {
    /// Initialize Fossology with URI and token.
    ///
    /// # Errors
    ///
    /// - API version can't be retrieved.
    pub fn new(uri: &str, token: &str) -> Result<Self, FossologyError> {
        let version = Self::version(uri, token)?;
        let fossology = Self {
            uri: uri.to_owned(),
            token: token.to_owned(),
            client: Client::new(),
            version,
        };

        Ok(fossology)
    }

    fn version(uri: &str, token: &str) -> Result<String, FossologyError> {
        let client = Client::new();
        let info = client
            .get(&format!("{}/info", uri))
            .bearer_auth(token)
            .send()?
            .json::<ApiInformation>();
        if let Ok(info) = info {
            Ok(info.version)
        } else {
            let version = client
                .get(&format!("{}/version", uri))
                .send()?
                .json::<ApiInformationV1>();
            match version {
                Ok(version) => Ok(version.version),
                Err(err) => Err(FossologyError::Other(err.to_string())),
            }
        }
    }

    pub(crate) fn version_is_at_least(&self, version: &str) -> Result<bool, FossologyError> {
        VersionCompare::compare_to(&self.version, version, &CompOp::Ge)
            .map_err(|_| FossologyError::Other("Failed to compare versions".to_string()))
    }

    pub(crate) fn init_get_with_token(&self, path: &str) -> RequestBuilder {
        self.client
            .get(&format!("{}/{}", self.uri, path))
            .bearer_auth(&self.token)
    }

    pub(crate) fn init_get(&self, path: &str) -> RequestBuilder {
        self.client.get(&format!("{}/{}", self.uri, path))
    }

    pub(crate) fn init_post_with_token(&self, path: &str) -> RequestBuilder {
        self.client
            .post(&format!("{}/{}", self.uri, path))
            .bearer_auth(&self.token)
    }
}

#[cfg(test)]
mod tests {
    use version_compare::{CompOp, VersionCompare};

    use super::Fossology;

    #[test]
    fn fossology_is_created() {
        let fossology = Fossology::new("http://localhost:8080/repo/api/v1", "token").unwrap();

        assert_eq!(fossology.token, "token");
        assert!(VersionCompare::compare_to(&fossology.version, "1.0.0", &CompOp::Ge).unwrap());
        assert!(VersionCompare::compare_to(&fossology.version, "2.0.0", &CompOp::Lt).unwrap());
    }
}
