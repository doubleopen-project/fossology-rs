use std::path::Path;

use reqwest::blocking::multipart::Form;
use serde::Deserialize;

use crate::{Fossology, FossologyError, FossologyResponse, InfoWithNumber};

pub fn new_upload_from_file<P: AsRef<Path>>(
    fossology: &Fossology,
    folder_id: i32,
    path_to_file: P,
) -> Result<FossologyResponse<NewUpload>, FossologyError> {
    let form = Form::new().file("fileInput", &path_to_file)?;

    let response = fossology
        .client
        .post(&format!("{}/uploads", fossology.uri))
        .bearer_auth(&fossology.token)
        .header("folderId", folder_id.to_string())
        .multipart(form)
        .send()?
        .json::<InfoWithNumber>()?;

    let new_upload = NewUpload {
        upload_id: response.message,
    };

    Ok(FossologyResponse::Response(new_upload))
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

#[derive(Deserialize)]
pub struct Hash {
    pub sha1: String,

    pub md5: String,

    pub sha256: String,

    pub size: i32,
}

#[cfg(test)]
mod test {
    use crate::auth::test::create_test_fossology_with_writetoken;

    use super::*;

    #[test]
    fn create_upload_from_file() {
        let fossology = create_test_fossology_with_writetoken("http://localhost:8080/repo/api/v1");

        let upload =
            new_upload_from_file(&fossology, 1, "tests/data/base-files_11.tar.xz").unwrap();

        match upload {
            FossologyResponse::Response(_) => {}
            FossologyResponse::ApiError(_) => panic!(),
        }
    }
}
