# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.1](https://github.com/avsaase/typenv/compare/typenv-v0.0.0...typenv-v0.0.1) - 2026-11-04

### Other

- Fix version pin
- Set version to 0.0.1
- Fix clippy
- Install nightly toolchain in CI
- Add README.md
- Merge CI workflows into one
- Add release-plz workflow
- Add GitHub Actions workflow for checks
- Add license files and add Cargo.toml metadata
- Rename crate from-env -> typenv
- Rustfmt
- Doc comment for derive macro and reformat comments
- Allow any error type that can be converted into BoxError for parsers
- Add inherent from_env method for structs deriving FromEnv
- Add docs for parser and error modules and hide private module from docs
- Remove reexports of parser modules
- Make error messages consistent
- Mark FromEnvError #[non_exhaustive]
- Add support for default_value = expr
- Add error for invalid defaults
- Error improvements
- Add support for prefixes
- Initial commit
- Add README to derive crate pointing to main crate
