<!--
SPDX-FileCopyrightText: 2021 HH Partners
 
SPDX-License-Identifier: MIT
 -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Return `None` instead of error from `get_upload_by_id()` if the request is otherwise successful
  but no upload with the given id exists.

### Fixed

- Change `assignee` field in upload to `Option` as it is not supported on older versions of
  Fossology API
- Return empty `Vec` instead of error if files are not found with filesearch.

[unreleased]: https://github.com/doubleopen-project/fossology-rs/compare/v0.2.0...HEAD
