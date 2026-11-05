# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/avsaase/typenv/compare/typenv-v0.0.3...typenv-v0.1.0) - 2026-11-05

### Other

- First release ready for use

## [0.0.3](https://github.com/avsaase/typenv/compare/typenv-v0.0.2...typenv-v0.0.3) - 2026-11-05

### Added

- [**breaking**] make FromEnv "sealed"
- automatically parse using FromStr, TryFrom<String> and From<String>
- [**breaking**] add ENV_VARS const to generated code describing the variables that will be read

## [0.0.2](https://github.com/avsaase/typenv/compare/typenv-v0.0.1...typenv-v0.0.2) - 2026-11-04

### Added

- add parser that uses serde to deserialize env var string value

## [0.0.1](https://github.com/avsaase/typenv/compare/typenv-v0.0.0...typenv-v0.0.1) - 2026-11-04

Initial release
