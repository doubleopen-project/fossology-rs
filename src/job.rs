// SPDX-FileCopyrightText: 2021 HH Partners
//
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

use crate::{Fossology, FossologyError, FossologyResponse, InfoWithNumber};

pub fn get_jobs(
    fossology: &Fossology,
    upload_id: Option<i32>,
    group_name: Option<String>,
    limit: Option<i32>,
    page: Option<i32>,
) -> Result<FossologyResponse<Vec<Job>>, FossologyError> {
    let mut builder = fossology.init_get_with_token("jobs");

    builder = if let Some(upload_id) = upload_id {
        builder.query(&[("upload", &upload_id.to_string())])
    } else {
        builder
    };

    builder = if let Some(group_name) = group_name {
        builder.header("groupName", group_name)
    } else {
        builder
    };

    builder = if let Some(limit) = limit {
        builder.header("limit", limit)
    } else {
        builder
    };

    builder = if let Some(page) = page {
        builder.header("page", page)
    } else {
        builder
    };

    let response = builder.send()?;

    Ok(response.json()?)
}

pub fn schedule_analysis(
    fossology: &Fossology,
    folder_id: i32,
    upload_id: i32,
    group_name: Option<String>,
    analysis: &ScheduleAgents,
) -> Result<FossologyResponse<ScheduledJob>, FossologyError> {
    let mut builder = fossology.init_post_with_token("jobs").json(analysis);

    builder = if let Some(group_name) = group_name {
        builder.header("groupName", group_name)
    } else {
        builder
    };

    let response = builder
        .header("folderId", folder_id.to_string())
        .header("uploadId", upload_id.to_string())
        .json(analysis)
        .send()?
        .json::<InfoWithNumber>()?;

    Ok(FossologyResponse::Response(ScheduledJob {
        id: response.message,
    }))
}

#[derive(Debug, Serialize)]
pub struct ScheduledJob {
    pub id: i32,
}

#[derive(Debug, Serialize, Default)]
pub struct ScheduleAgents {
    pub analysis: Analysis,
    pub decider: Decider,
    pub reuse: Reuse,
}

#[derive(Debug, Serialize, Default)]
pub struct Analysis {
    pub bucket: bool,
    pub copyright_email_author: bool,
    pub ecc: bool,
    pub keyword: bool,
    pub mime: bool,
    pub monk: bool,
    pub nomos: bool,
    pub ojo: bool,
    pub package: bool,
}

#[derive(Debug, Serialize, Default)]
pub struct Decider {
    pub nomos_monk: bool,
    /// Needs to be false for the other deciders to work:
    /// https://github.com/fossology/fossology/issues/1639
    bulk_reused: bool,
    pub new_scanner: bool,
    pub ojo_decider: bool,
}

#[derive(Debug, Serialize, Default)]
pub struct Reuse {
    pub reuse_upload: i32,
    pub reuse_group: String,
    pub reuse_main: bool,
    pub reuse_enhanced: bool,
    pub reuse_report: bool,
    pub reuse_copyright: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: i32,

    pub name: String,

    pub queue_date: String,

    pub upload_id: String,

    pub user_id: String,

    pub group_id: String,

    pub eta: i32,

    pub status: JobStatus,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub enum JobStatus {
    Completed,
    Failed,
    Queued,
    Processing,
}

#[cfg(test)]
mod test {
    use std::{thread, time::Duration};

    use crate::{auth::test::create_test_fossology_with_writetoken, upload::new_upload_from_file};

    use super::*;

    #[test]
    fn get_unarchive_job() {
        let fossology = create_test_fossology_with_writetoken("http://localhost:8080/repo/api/v1");

        let upload = new_upload_from_file(&fossology, 1, "tests/data/base-files_11.tar.xz")
            .unwrap()
            .response_unchecked();

        let jobs = get_jobs(&fossology, Some(upload.upload_id), None, None, None)
            .unwrap()
            .response_unchecked();

        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].status, JobStatus::Processing)
    }

    #[test]
    fn schedule_jobs() {
        let fossology = create_test_fossology_with_writetoken("http://localhost:8080/repo/api/v1");

        let upload = new_upload_from_file(&fossology, 1, "tests/data/base-files_11.tar.xz")
            .unwrap()
            .response_unchecked();

        let jobs = get_jobs(&fossology, Some(upload.upload_id), None, None, None)
            .unwrap()
            .response_unchecked();

        assert_eq!(jobs.len(), 1);

        while get_jobs(&fossology, Some(upload.upload_id), None, None, None)
            .unwrap()
            .response_unchecked()[0]
            .status
            == JobStatus::Processing
        {
            thread::sleep(Duration::from_secs(1));
        }

        let mut schedule = ScheduleAgents::default();
        schedule.analysis.nomos = true;
        schedule.analysis.ojo = true;
        schedule.analysis.copyright_email_author = true;
        schedule.analysis.ecc = true;
        schedule.analysis.keyword = true;

        let scheduled_job = schedule_analysis(&fossology, 1, upload.upload_id, None, &schedule)
            .unwrap()
            .response_unchecked();

        let jobs = get_jobs(&fossology, Some(upload.upload_id), None, None, None)
            .unwrap()
            .response_unchecked();

        assert_eq!(jobs.len(), 2);
        assert!(jobs.iter().any(|j| j.id == scheduled_job.id));
    }
}
