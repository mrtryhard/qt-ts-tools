# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [0.5.0] - unreleased

### Added

- The tool now support logging for debugging and observability purpose [#18](https://github.com/mrtryhard/qt-ts-tools/issues/18)

### Changed

### Fixed

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

## Fixed

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
