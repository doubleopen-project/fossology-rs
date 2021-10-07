// SPDX-FileCopyrightText: 2021 HH Partners
//
// SPDX-License-Identifier: MIT

use std::path::Path;

use reqwest::blocking::multipart::Form;
use serde::{Deserialize, Serialize};

use crate::{Fossology, FossologyError, FossologyResponse, InfoWithNumber};

/// # Errors
///
/// - File can't be opened.
/// - Error while sending request, redirect loop was detected or redirect limit was exhausted.
/// - Response can't be serialized to [`InfoWithNumber`] or [`Info`](crate::Info).
/// - Response is not [`InfoWithNumber`].
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

/// # Errors
///
/// - Error while sending request, redirect loop was detected or redirect limit was exhausted.
/// - Response can't be serialized to [`Upload`] or [`Info`](crate::Info).
/// - Response is not [`Upload`].
pub fn get_upload_by_id(
    fossology: &Fossology,
    upload_id: i32,
) -> Result<Option<Upload>, FossologyError> {
    let response = fossology
        .client
        .get(&format!("{}/uploads/{}", fossology.uri, upload_id))
        .bearer_auth(&fossology.token)
        .send()?
        .json::<FossologyResponse<Upload>>()?;

    match response {
        FossologyResponse::Response(res) => Ok(Some(res)),
        FossologyResponse::ApiError(err) => {
            if err.message == "Upload does not exist" {
                Ok(None)
            } else {
                Err(FossologyError::Other(err.message))
            }
        }
    }
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

    #[serde(default)]
    pub assignee: Option<i32>,

    pub hash: Hash,
}

/// # Errors
///
/// - Error while sending request, redirect loop was detected or redirect limit was exhausted.
/// - Response can't be serialized to [`Vec`] of [`FilesearchResponse`]s or [`Info`](crate::Info).
/// - Response is not [`Vec`] of [`FilesearchResponse`]s.
pub fn filesearch(
    fossology: &Fossology,
    hashes: &[Hash],
    group_name: Option<String>,
) -> Result<Vec<FilesearchResponse>, FossologyError> {
    let mut builder = fossology.init_post_with_token("filesearch").json(hashes);

    builder = if let Some(group_name) = group_name {
        builder.header("groupName", group_name)
    } else {
        builder
    };

    let response = builder.send()?;

    let response = response.json::<FossologyResponse<Vec<FilesearchResponse>>>()?;
    match response {
        FossologyResponse::Response(res) => {
            let res = res
                .into_iter()
                .filter(|i| i.message != Some("Not found".to_string()))
                .collect();
            Ok(res)
        }
        FossologyResponse::ApiError(err) => Err(FossologyError::Other(err.message)),
    }
}

#[derive(Deserialize, Debug)]
pub struct FilesearchResponse {
    pub hash: Hash,
    pub findings: Option<Findings>,
    #[serde(default)]
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
    use std::{thread, time::Duration};

    use crate::{
        auth::test::create_test_fossology_with_writetoken,
        job::{get_jobs, JobStatus},
        utilities::hash256_for_path,
    };

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

        let filesearch = filesearch(&fossology, &hashes, None).unwrap();

        assert!(filesearch[0].uploads.contains(&upload.upload_id));
    }

    #[test]
    fn upload_is_retrievable_by_id() {
        let fossology = create_test_fossology_with_writetoken("http://localhost:8080/repo/api/v1");

        let upload =
            new_upload_from_file(&fossology, 1, "tests/data/base-files_11.tar.xz").unwrap();

        while get_jobs(&fossology, Some(upload.upload_id), None, None, None).unwrap()[0].status
            == JobStatus::Processing
        {
            thread::sleep(Duration::from_secs(1));
        }

        let upload = get_upload_by_id(&fossology, upload.upload_id)
            .unwrap()
            .unwrap();

        assert_eq!(upload.folder_id, 1);
    }

    #[test]
    fn non_existing_upload_id_returns_none() {
        let fossology = create_test_fossology_with_writetoken("http://localhost:8080/repo/api/v1");

        let upload = get_upload_by_id(&fossology, 99999).unwrap();

        assert!(upload.is_none());
    }

    #[test]
    fn non_existing_hash_for_filesearch_works() {
        let fossology = create_test_fossology_with_writetoken("http://localhost:8080/repo/api/v1");

        let hashes = vec![Hash::from_sha256("doesnotexist")];
        let filesearch = filesearch(&fossology, &hashes, None).unwrap();

        assert!(filesearch.is_empty());
    }
}
