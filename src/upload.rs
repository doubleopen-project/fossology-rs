// SPDX-FileCopyrightText: 2021 HH Partners
//
// SPDX-License-Identifier: MIT

use std::path::Path;

use reqwest::blocking::multipart::Form;
use serde::{Deserialize, Serialize};

use crate::{Fossology, FossologyError, FossologyResponse, InfoWithNumber};

pub fn new_upload_from_file<P: AsRef<Path>>(
    fossology: &Fossology,
    folder_id: i32,
    path_to_file: P,
) -> Result<NewUpload, FossologyError> {
    let form = Form::new().file("fileInput", &path_to_file)?;

    let response = fossology
        .client
        .post(&format!("{}/uploads", fossology.uri))
        .bearer_auth(&fossology.token)
        .header("folderId", folder_id.to_string())
        .multipart(form)
        .send()?
        .json::<FossologyResponse<InfoWithNumber>>()?;

    match response {
        FossologyResponse::Response(res) => Ok(NewUpload {
            upload_id: res.message,
        }),
        FossologyResponse::ApiError(err) => Err(FossologyError::Other(err.message)),
    }
}

pub fn get_upload_by_id(
    fossology: &Fossology,
    upload_id: i32,
) -> Result<FossologyResponse<Upload>, FossologyError> {
    let response = fossology
        .client
        .get(&format!("{}/uploads/{}", fossology.uri, upload_id))
        .bearer_auth(&fossology.token)
        .send()?
        .json::<FossologyResponse<Upload>>()?;

    Ok(response)
}

pub struct NewUpload {
    pub upload_id: i32,
}

#[derive(Deserialize)]
pub struct Upload {
    #[serde(rename = "folderid")]
    pub folder_id: i32,

    #[serde(rename = "foldername")]
    pub folder_name: String,

    pub id: i32,

    pub description: String,

    #[serde(rename = "uploadname")]
    pub upload_name: String,

    #[serde(rename = "uploaddate")]
    pub upload_date: String,

    pub assignee: i32,

    pub hash: Hash,
}

pub fn filesearch(
    fossology: &Fossology,
    hashes: &[Hash],
    group_name: Option<String>,
) -> Result<FossologyResponse<Vec<FilesearchResponse>>, FossologyError> {
    let mut builder = fossology.init_post_with_token("filesearch").json(hashes);

    builder = if let Some(group_name) = group_name {
        builder.header("groupName", group_name)
    } else {
        builder
    };

    let response = builder.send()?;
    let response = response.json::<FossologyResponse<Vec<FilesearchResponse>>>()?;
    Ok(response)
}

#[derive(Deserialize, Debug)]
pub struct FilesearchResponse {
    pub hash: Hash,
    pub findings: Option<Findings>,
    pub uploads: Vec<i32>,
    pub message: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Findings {
    pub scanner: Vec<String>,
    pub conclusion: Vec<String>,
    pub copyright: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Hash {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub sha1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub md5: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub size: Option<i32>,
}

impl Hash {
    pub fn from_sha1(sha1: &str) -> Self {
        Self {
            sha1: Some(sha1.to_string()),
            ..Self::default()
        }
    }

    pub fn from_sha256(sha256: &str) -> Self {
        Self {
            sha256: Some(sha256.to_string()),
            ..Self::default()
        }
    }

    pub fn from_md5(md5: &str) -> Self {
        Self {
            md5: Some(md5.to_string()),
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{auth::test::create_test_fossology_with_writetoken, utilities::hash256_for_path};

    use super::*;

    #[test]
    fn create_upload_from_file() {
        let fossology = create_test_fossology_with_writetoken("http://localhost:8080/repo/api/v1");

        new_upload_from_file(&fossology, 1, "tests/data/base-files_11.tar.xz").unwrap();
    }

    #[test]
    fn filesearch_for_archive() {
        let fossology = create_test_fossology_with_writetoken("http://localhost:8080/repo/api/v1");
        let sha256 = hash256_for_path("tests/data/base-files_11.tar.xz");

        let upload =
            new_upload_from_file(&fossology, 1, "tests/data/base-files_11.tar.xz").unwrap();

        let hashes = vec![Hash::from_sha256(&sha256)];

        let filesearch = filesearch(&fossology, &hashes, None)
            .unwrap()
            .response_unchecked();

        assert!(filesearch[0].uploads.contains(&upload.upload_id));
    }
}
