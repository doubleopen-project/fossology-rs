// SPDX-FileCopyrightText: 2021 HH Partners
//
// SPDX-License-Identifier: MIT

use serde::Deserialize;

use crate::{Fossology, FossologyError, FossologyResponse};

/// # Errors
///
/// - Error while sending request, redirect loop was detected or redirect limit was exhausted.
/// - Response can't be serialized to [`ApiInformation`] or [`Info`](crate::Info).
/// - Response is not [`ApiInformation`].
pub fn info(fossology: &Fossology) -> Result<ApiInformation, FossologyError> {
    let response: FossologyResponse<ApiInformation> = fossology
        .client
        .get(&format!("{}/info", fossology.uri))
        .send()?
        .json()?;

    match response {
        FossologyResponse::Response(res) => Ok(res),
        FossologyResponse::ApiError(err) => Err(FossologyError::Other(err.message)),
    }
}

#[derive(Debug, Deserialize)]
pub struct ApiInformation {
    pub name: String,
    pub description: String,
    pub version: String,
    pub security: Vec<String>,
    pub contact: String,
    pub license: ApiLicense,
}

#[derive(Debug, Deserialize)]
pub struct ApiLicense {
    pub name: String,
    pub url: String,
}

/// # Errors
///
/// - Error while sending request, redirect loop was detected or redirect limit was exhausted.
/// - Response can't be serialized to [`Health`] or [`Info`](crate::Info).
/// - Response is not [`Health`].
pub fn health(fossology: &Fossology) -> Result<Health, FossologyError> {
    let response: FossologyResponse<Health> = fossology
        .client
        .get(&format!("{}/health", fossology.uri))
        .send()?
        .json()?;
    match response {
        FossologyResponse::Response(res) => Ok(res),
        FossologyResponse::ApiError(res) => Err(FossologyError::Other(res.message)),
    }
}

#[derive(Debug, Deserialize)]
pub struct Health {
    pub status: String,
    pub scheduler: Status,
    pub db: Status,
}

#[derive(Debug, Deserialize)]
pub struct Status {
    pub status: String,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn api_information() {
        let fossology = Fossology::new("http://localhost:8080/repo/api/v1", "token");

        let info = info(&fossology).unwrap();

        assert_eq!(info.name, "FOSSology API");
    }

    #[test]
    fn get_health() {
        let fossology = Fossology::new("http://localhost:8080/repo/api/v1", "token");

        let health = health(&fossology).unwrap();

        assert_eq!(health.status, "OK");
        assert_eq!(health.scheduler.status, "OK");
        assert_eq!(health.db.status, "OK");
    }
}
