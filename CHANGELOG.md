<!--
SPDX-FileCopyrightText: 2021 HH Partners
 
SPDX-License-Identifier: MIT
 -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.2] - 2021-11-30

### Fixed

- Increase default timeout duration to 600 seconds.

## [0.2.1] - 2021-10-07

### Fixed

- **BREAKING**: Change `assignee` field in upload to `Option` as it is not supported on older
  versions of Fossology API.
- Return empty `Vec` instead of error if files are not found with filesearch.
- Return `None` instead of error from `get_upload_by_id()` if the request is otherwise successful
  but no upload with the given id exists.

[unreleased]: https://github.com/doubleopen-project/fossology-rs/compare/v0.2.2...HEAD
[0.2.1]: https://github.com/doubleopen-project/fossology-rs/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/doubleopen-project/fossology-rs/compare/v0.2.0...v0.2.1
