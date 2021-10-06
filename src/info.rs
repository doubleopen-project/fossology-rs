use serde::Deserialize;

use crate::{Fossology, FossologyError, FossologyResponse};

pub fn info(fossology: &Fossology) -> Result<FossologyResponse<ApiInformation>, FossologyError> {
    let response: FossologyResponse<ApiInformation> = fossology
        .client
        .get(&format!("{}/info", fossology.uri))
        .send()?
        .json()?;
    Ok(response)
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

pub fn health(fossology: &Fossology) -> Result<FossologyResponse<Health>, FossologyError> {
    let response: FossologyResponse<Health> = fossology
        .client
        .get(&format!("{}/health", fossology.uri))
        .send()?
        .json()?;
    Ok(response)
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

        if let FossologyResponse::Response(response) = info {
            assert_eq!(response.name, "FOSSology API");
        } else {
            panic!("No response");
        }
    }

    #[test]
    fn get_health() {
        let fossology = Fossology::new("http://localhost:8080/repo/api/v1", "token");

        let info = health(&fossology).unwrap();

        if let FossologyResponse::Response(response) = info {
            assert_eq!(response.status, "OK");
            assert_eq!(response.scheduler.status, "OK");
            assert_eq!(response.db.status, "OK");
        } else {
            panic!("No response");
        }
    }
}
