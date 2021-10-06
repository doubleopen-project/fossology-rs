use serde::Deserialize;

use crate::{Fossology, FossologyError, FossologyResponse};

pub fn get_license(
    fossology: &Fossology,
    short_name: &str,
    group_name: Option<&str>,
) -> Result<License, FossologyError> {
    let mut builder = fossology.init_get_with_token(&format!("license/{}", short_name));

    builder = if let Some(group_name) = group_name {
        builder.header("groupName", group_name)
    } else {
        builder
    };

    let response = builder.send()?;

    let bytes = response.bytes()?;

    let response = serde_json::from_slice::<FossologyResponse<License>>(&bytes);

    match response {
        Ok(foss_res) => match foss_res {
            FossologyResponse::Response(res) => Ok(res),
            FossologyResponse::ApiError(err) => Err(FossologyError::Other(err.message)),
        },
        Err(_) => Err(FossologyError::UnexpectedResponse(
            String::from_utf8_lossy(&bytes).to_string(),
        )),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct License {
    pub id: i32,
    pub short_name: String,
    pub full_name: String,
    pub text: String,
    #[serde(default)]
    pub risk: Option<i32>,
    pub is_candidate: bool,
}

#[cfg(test)]
mod test {
    use crate::auth::test::create_test_fossology_with_writetoken;

    use super::*;

    #[test]
    fn get_correct_license() {
        let fossology = create_test_fossology_with_writetoken("http://localhost:8080/repo/api/v1");

        let mit = get_license(&fossology, "MIT", None).unwrap();

        assert_eq!(mit.full_name, "MIT License");
    }

    #[test]
    fn error_on_invalid_license() {
        let fossology = create_test_fossology_with_writetoken("http://localhost:8080/repo/api/v1");

        let err = get_license(&fossology, "does_not_exist", None).unwrap_err();

        assert!(err.to_string().contains("No license found"));
    }
}
