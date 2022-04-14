<!--
SPDX-FileCopyrightText: 2021 HH Partners
 
SPDX-License-Identifier: MIT
 -->

# Changelog

## [0.2.3] - 2022-04-13

### Added

- Implement `Eq`, `PartialEq` and `Hash` for `Hash` struct.

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

[0.2.3]: https://github.com/doubleopen-project/fossology-rs/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/doubleopen-project/fossology-rs/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/doubleopen-project/fossology-rs/compare/v0.2.0...v0.2.1
