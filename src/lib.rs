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
use std::time::Duration;
use version_compare::{CompOp, VersionCompare};

use crate::info::{ApiInformation, ApiInformationV1};

pub mod auth;
pub mod info;
pub mod job;
pub mod license;
pub mod upload;
mod utilities;

/// Client for the Fossology API.
#[derive(Debug)]
pub struct Fossology {
    /// API base uri.
    uri: String,

    /// Access token for Fossology.
    token: String,

    /// Reqwest client.
    client: Client,

    /// Version of the Fossology API. Is retrieved during creation.
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
enum FossologyResponse<T> {
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
    /// Creates a client for Fossology API.
    ///
    /// Gets the version of the API during creation. The version is used to guard for endpoints that
    /// are not supported in the version being accessed.
    ///
    /// # Errors
    ///
    /// - API version can't be retrieved.
    pub fn new(uri: &str, token: &str) -> Result<Self, FossologyError> {
        let version = Self::version(uri, token)?;
        let client = Client::builder()
            .timeout(Duration::from_secs(600))
            .build()?;
        let fossology = Self {
            uri: uri.to_owned(),
            token: token.to_owned(),
            client,
            version,
        };

        Ok(fossology)
    }

    /// Get the version of the API. Tries different endpoints to get version for older and newer
    /// instances.
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

    /// Returns true if the API version is at least the given version.
    pub(crate) fn version_is_at_least(&self, version: &str) -> Result<bool, FossologyError> {
        VersionCompare::compare_to(&self.version, version, &CompOp::Ge)
            .map_err(|_| FossologyError::Other("Failed to compare versions".to_string()))
    }

    /// Initializes `GET` request with the authorization token.
    pub(crate) fn init_get_with_token(&self, path: &str) -> RequestBuilder {
        self.client
            .get(&format!("{}/{}", self.uri, path))
            .bearer_auth(&self.token)
    }

    /// Initializes `GET` request without the authorization token.
    pub(crate) fn init_get(&self, path: &str) -> RequestBuilder {
        self.client.get(&format!("{}/{}", self.uri, path))
    }

    /// Initializes `POST` request with the authorization token.
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
