// SPDX-FileCopyrightText: 2021 HH Partners
//
// SPDX-License-Identifier: MIT

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::{Fossology, FossologyError, FossologyResponse};

/// # Errors
///
/// - Error while sending request, redirect loop was detected or redirect limit was exhausted.
/// - Response can't be serialized to [`Token`] or [`Info`](crate::Info).
/// - Response is not [`Token`].
pub fn tokens(fossology: &Fossology, params: &TokensParameters) -> Result<Token, FossologyError> {
    let response = fossology
        .client
        .post(&format!("{}/tokens", fossology.uri))
        .json(&params)
        .send()?
        .json::<FossologyResponse<Token>>()?;

    match response {
        FossologyResponse::Response(res) => Ok(res),
        FossologyResponse::ApiError(err) => Err(FossologyError::Other(err.message)),
    }
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

    pub fn create_test_fossology_with_writetoken(uri: &str) -> Fossology {
        let fossology = Fossology::new(uri, "token").unwrap();
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

        let token = tokens(&fossology, &params).unwrap();

        Fossology::new(
            "http://localhost:8080/repo/api/v1",
            token.authorization.strip_prefix("Bearer ").unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn generate_read_token() {
        let fossology = Fossology::new("http://localhost:8080/repo/api/v1", "token").unwrap();

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

        let token = tokens(&fossology, &params).unwrap();

        assert!(token.authorization.starts_with("Bearer"));
    }

    #[test]
    fn generate_write_token() {
        let fossology = Fossology::new("http://localhost:8080/repo/api/v1", "token").unwrap();

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

        let tokens = tokens(&fossology, &params).unwrap();

        assert!(tokens.authorization.starts_with("Bearer"));
    }
}
