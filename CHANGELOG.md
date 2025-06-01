# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [0.8.6] unreleased

### Changed

- Upgraded `i18n-embed-fl` to 0.6.0 and modified inner `tr!` macro to simplify and improve compatibility with `i18n-embed-fl` `fl!` macro.

## [0.8.5] 2025-05-09
[Fourteenth milestone](https://github.com/mrtryhard/qt-ts-tools/milestone/15).
This release is a maintenance release and introduces an upgrade of dependencies

### Changed

- Upgraded `clap` to version `4.5.37`
- Upgraded `clap_complete` to version `4.5.50`
- Upgraded `i18n-embed` to version `0.15.4`
- Upgraded `i18n-embed-fl` to version `0.9.4`
- Upgraded `quick-xml` to version `0.37.5`
- Upgraded `rust-embed` to version `8.7.1`

## [0.8.4] 2025-04-04
[Thirteenth milestone](https://github.com/mrtryhard/qt-ts-tools/milestone/14).
This release is a maintenance release and introduces an upgrade of dependencies

### Fixed

- Updated executable metadata to report the right version [#209](https://github.com/mrtryhard/qt-ts-tools/issues/175)

### Changed

- Upgraded `clap` to version `4.5.35`
- Upgraded `clap_complete` to version `4.5.47`
- Upgraded `env_logger` to version `0.11.8`
- Upgraded `log` to version `0.4.27`
- Upgraded `quick-xml` to version `0.37.3`
- Upgraded `serde` to version `1.0.219`

## [0.8.3] 2025-03-01
[Thirteenth milestone](https://github.com/mrtryhard/qt-ts-tools/milestone/13).
This release is a maintenance release and introduces an upgrade of dependencies and Rust edition. 

### Changed

- Rust edition upgraded from 2021 to 2024 [#175](https://github.com/mrtryhard/qt-ts-tools/issues/175)
- Upgraded `clap` to version `4.5.31`
- Upgraded `clap_complete` to version `4.5.46`
- Upgraded `log` to version `0.4.26`
- Upgraded `rust_embed` to version `8.6.0`
- Upgraded `serde` to version `1.0.218`

## [0.8.2] 2025-01-18
[Twelveth milestone](https://github.com/mrtryhard/qt-ts-tools/milestone/12).
This release introduces ARM64 for Linux and minor changes.

### Added

- Added arm64 Linux binary in release [#195](https://github.com/mrtryhard/qt-ts-tools/issues/195)

### Changed

- Upgraded `clap` to version `4.5.26`
- Upgraded `clap_complete` to version `4.5.42`
- Upgraded `log` to version `0.4.25`

## [0.8.1] - 2025-01-08
[Eleventh milestone](https://github.com/mrtryhard/qt-ts-tools/milestone/11).  
This releases is a maintenance release and contains dependencies updates.

### Changed

- Updated `clap` to version `4.5.24`
- Updated `clap_complete` to version `4.5.41`
- Updated `clap_complete_nushell` to version `4.5.5`
- Updated `env_logger` to version `0.11.6`
- Updated `i18n-embed` to version `0.15.3`
- Updated `i18n-embed-fl` to version `0.9.3`
- Updated `quick-xml` to version `0.37.2`
- Updated `serde` to version `1.0.217`

## [0.8.0] - 2024-11-20
[Tenth milestone](https://github.com/mrtryhard/qt-ts-tools/milestone/10).
This release contains breaking changes on the output produce by the tool. Otherwise, mostly dependencies upgrades.

### Added

- Added MacOS build binary in the official release. This build is not officially tested, but should work. [#40](https://github.com/mrtryhard/qt-ts-tools/issues/40)

### Fixed

- Fixes security advisory check for unmaintained `proc-macro-error` crate indirectly linked with `i18n-embed` and `i18n-embed-fl` dependencies. [#157](https://github.com/mrtryhard/qt-ts-tools/issues/157)

### Changed

- Updated `clap` to version `4.5.21`
- Updated `clap_complete` to version `4.5.38`
- Updated `clap_complete_nushell` to version `4.5.4`
- Updated `i18n-embed` to version `0.15.2`
- Updated `i18n-embed-fl` to version `0.9.2`
- Updated `serde` to version `1.0.215`
- Updated `sys-locale` to version `0.3.2``
- Breaking: Updated `quick-xml` to `0.37.1`, which effectively changes the output produced by the tool. 
  The `translation` nodes now will have no indent. [#170](https://github.com/mrtryhard/qt-ts-tools/issues/170)

## [0.7.0] - 2024-08-06
[Ninth milestone](https://github.com/mrtryhard/qt-ts-tools/milestone/9).
This release improves on some missing functionality and functionalities defaults. Since it is breaking changes, 
the version gets bumped to 0.7.0.

### Added

- Added `--keep-translation` flag to `merge` command to ensure to not override existing translations. 
  Useful when you want to add new strings upon existing ones [#132](https://github.com/mrtryhard/qt-ts-tools/issues/132)

### Fixed

- Fixed default output for `stat` command. Now defaults to short summary. Extended stats can be obtained with the `verbose` flag [#129](https://github.com/mrtryhard/qt-ts-tools/issues/129)
- Fixed output for detailed report with very long file paths. The format takes more space but is more readable [#130](https://github.com/mrtryhard/qt-ts-tools/issues/130)

### Changed

- Removed dependency on `itertools` in [#132](https://github.com/mrtryhard/qt-ts-tools/issues/132)
- Updated `clap` to version 4.5.13 (no ticket)
- Updated `clap_complete` to version 4.5.13 (no ticket)

## [0.6.0] - 2024-07-28
[Eight milestone](https://github.com/mrtryhard/qt-ts-tools/milestone/8).
This release makes the tool feature complete. The tool remains open for new features and bug fixing.

### Added

- Added `stat` command line to query information about the input file [#111](https://github.com/mrtryhard/qt-ts-tools/issues/111)

### Fixed

- Fixed reference to non-existing translation for `stat` command [#96](https://github.com/mrtryhard/qt-ts-tools/issues/96)
- Executable size was 1.7MB in 0.5.1, 3.2MB during 0.6.0 development, now 1.2MB [#123](https://github.com/mrtryhard/qt-ts-tools/issues/123)

### Changed

- Now use `i18n_embed` for localization [#91](https://github.com/mrtryhard/qt-ts-tools/issues/91)
- Now validate missing or incorrect translation during compilation [#96](https://github.com/mrtryhard/qt-ts-tools/issues/96)
- Updated `clap` to version 4.5.11 (no ticket)
- Updated `clap_complete` to version 4.5.11 (no ticket)
- Updated `clap_complete_command` to 0.6.1 (no ticket)
- Updated `quick_xml` to 0.36.1 (no ticket)

## [0.5.1] - 2024-06-29
[Seventh milestone](https://github.com/mrtryhard/qt-ts-tools/milestone/7). 
This release aims to make the executable even more robust, fix bugs and regressions.

### Changed

- Log library changed to `env_logger` for the purpose of reducing executable size [#95](https://github.com/mrtryhard/qt-ts-tools/issues/95)
- Improved automated tests [#21](https://github.com/mrtryhard/qt-ts-tools/issues/21)
- Updated third party `quick-xml` to 0.34.0 (no ticket)
- Updated third party `clap_complete` to 4.5.8 (no ticket)
- Updated third party `log` to 0.4.22 (no ticket)
- Removed third party `lazy-static` and replaced with standard library `OnceLock` [#108](https://github.com/mrtryhard/qt-ts-tools/issues/21)
- Simplified translation usage by unifying to a single variable macro `tr!` [#113](https://github.com/mrtryhard/qt-ts-tools/issues/113)

### Fixed

- The `sort` command now sorts context-less `message` nodes correctly [#99](https://github.com/mrtryhard/qt-ts-tools/issues/99)
- The `extract` command now extracts context-less `message` nodes correctly [#101](https://github.com/mrtryhard/qt-ts-tools/issues/101)

## [0.5.0] - 2024-06-22
[Sixth milestone](https://github.com/mrtryhard/qt-ts-tools/milestone/6). This release aims to polish what's been done so far.

### Added

- The tool now support logging for debugging and observability purpose [#18](https://github.com/mrtryhard/qt-ts-tools/issues/18)
- Added `output-path` to `shell-completion` command [#57](https://github.com/mrtryhard/qt-ts-tools/issues/57)
- Added `strip` command to strip specific types of translation out of a file [#56](https://github.com/mrtryhard/qt-ts-tools/issues/56)

### Changed

- Improved code documentation for reusable parsing TS structures [#41](https://github.com/mrtryhard/qt-ts-tools/issues/41)
- Extracted cli commands in their own subdirectory for code clarity [#57](https://github.com/mrtryhard/qt-ts-tools/issues/57)
- Updated clap-rs third party to 4.5.7 [#83](https://github.com/mrtryhard/qt-ts-tools/issues/83)
- Updated quick-xml third party to 0.32.0 [#85](https://github.com/mrtryhard/qt-ts-tools/issues/85)

### Fixed

- Fixed an issue where the `help` parameter was error-ing every command [#82](https://github.com/mrtryhard/qt-ts-tools/issues/82)

## [0.4.0] - 2024-05-24
[Fifth milestone](https://github.com/mrtryhard/qt-ts-tools/milestone/4). This release introduces some user experience improvement efforts such as localization (adopting current system language) and auto-completion support.

### Added

- Shell auto-completion scripts can now be auto-generated when using `shell-completion <shell name>` command line option. [#32](https://github.com/mrtryhard/qt-ts-tools/issues/32)
- Localization is now supported. French and English are supported [#28](https://github.com/mrtryhard/qt-ts-tools/issues/28), [#63](https://github.com/mrtryhard/qt-ts-tools/issues/28)

### Changed

- Fixed a lot of clippy errors, simplified the sorting algorithm for messages [#60](https://github.com/mrtryhard/qt-ts-tools/issues/60)
- Bumped Serde, Itertools versions [#72](https://github.com/mrtryhard/qt-ts-tools/issues/72)

### Fixed 

### Known issues

- Command lines _errors_ are not translated for now. [#73](https://github.com/mrtryhard/qt-ts-tools/issues/73)

## [0.3.1] - 2024-05-12
[Fourth milestone](https://github.com/mrtryhard/qt-ts-tools/milestone/5). Contains minor fixes to the command line tool.

### Fixed

- Sorting now consider messages' id [#42](https://github.com/mrtryhard/qt-ts-tools/issues/42)

## [0.3.0] - 2024-04-22
[Third milestone](https://github.com/mrtryhard/qt-ts-tools/milestone/3). This introduces the `merge` command and improved documentation.

### Added

- Merge mechanism to merge two translation files [#24](https://github.com/mrtryhard/qt-ts-tools/issues/24)
- `extra-*` fields support in `TS` and `message` nodes [#4](https://github.com/mrtryhard/qt-ts-tools/issues/4)

### Changed

- Improved command line documentation [#25](https://github.com/mrtryhard/qt-ts-tools/issues/25), [#27](https://github.com/mrtryhard/qt-ts-tools/issues/27)
- Updated Clap dependencies [#26](https://github.com/mrtryhard/qt-ts-tools/issues/26)
- Updated Serde dependencies

### Fixed

## [0.2.0] - 2024-01-01

Completion of the [second milestone](https://github.com/mrtryhard/qt-ts-tools/milestone/2). This introduces the `extract` and lighter binary.

### Added

- Extraction mechanism to extract only relevant translation types [#16](https://github.com/mrtryhard/qt-ts-tools/issues/16)

### Changed

- Reduced release binary size [#19](https://github.com/mrtryhard/qt-ts-tools/issues/19)
- Updated Serde and Clap dependencies [#20](https://github.com/mrtryhard/qt-ts-tools/issues/20)

### Fixed

## [0.1.0] - 2024-01-30

Introduction of `qt-ts-tools`. This completes the first [milestone](https://github.com/mrtryhard/qt-ts-tools/milestone/1?closed=1).

### Added

- Sort mechanism to sort translation files by location and contexts [#3](https://github.com/mrtryhard/qt-ts-tools/issues/3)
