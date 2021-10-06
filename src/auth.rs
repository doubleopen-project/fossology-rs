use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::{Fossology, FossologyError, FossologyResponse};

pub fn tokens(
    fossology: &Fossology,
    params: &TokensParameters,
) -> Result<FossologyResponse<Token>, FossologyError> {
    let response = fossology
        .client
        .post(&format!("{}/tokens", fossology.uri))
        .json(&params)
        .send()?
        .json()?;

    Ok(response)
}

#[derive(Debug, Serialize)]
pub struct TokensParameters {
    username: String,
    password: String,
    token_name: String,
    token_scope: TokenScope,
    token_expire: NaiveDate,
}

#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum TokenScope {
    Read,
    Write,
}

impl TokensParameters {
    pub fn new(
        username: &str,
        password: &str,
        token_name: &str,
        token_scope: TokenScope,
        token_expire: NaiveDate,
    ) -> Self {
        Self {
            username: username.to_string(),
            password: password.to_string(),
            token_name: token_name.to_string(),
            token_scope,
            token_expire,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Token {
    #[serde(rename = "Authorization")]
    pub authorization: String,
}

#[cfg(test)]
pub(crate) mod test {
    use rand::{distributions::Alphanumeric, Rng};

    use super::*;

    pub(crate) fn create_test_fossology_with_writetoken(uri: &str) -> Fossology {
        let fossology = Fossology::new(uri, "token");
        let token_name = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect::<String>();

        let params = TokensParameters::new(
            "fossy",
            "fossy",
            &token_name,
            TokenScope::Write,
            NaiveDate::from_ymd(2021, 10, 30),
        );

        let info = tokens(&fossology, &params).unwrap();

        match info {
            FossologyResponse::Response(response) => Fossology::new(
                "http://localhost:8080/repo/api/v1",
                response.authorization.strip_prefix("Bearer ").unwrap(),
            ),
            FossologyResponse::ApiError(_) => panic!(),
        }
    }

    #[test]
    fn generate_read_token() {
        let fossology = Fossology::new("http://localhost:8080/repo/api/v1", "token");

        let token_name = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect::<String>();

        let params = TokensParameters::new(
            "fossy",
            "fossy",
            &token_name,
            TokenScope::Read,
            NaiveDate::from_ymd(2021, 10, 30),
        );

        dbg!(&params.token_expire);

        let info = tokens(&fossology, &params).unwrap();

        if let FossologyResponse::Response(response) = info {
            assert!(response.authorization.starts_with("Bearer"));
        } else {
            panic!("No response");
        }
    }

    #[test]
    fn generate_write_token() {
        let fossology = Fossology::new("http://localhost:8080/repo/api/v1", "token");

        let token_name = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect::<String>();

        let params = TokensParameters::new(
            "fossy",
            "fossy",
            &token_name,
            TokenScope::Write,
            NaiveDate::from_ymd(2021, 10, 30),
        );

        let info = tokens(&fossology, &params).unwrap();

        if let FossologyResponse::Response(response) = info {
            assert!(response.authorization.starts_with("Bearer"));
        } else {
            panic!("No response");
        }
    }
}
