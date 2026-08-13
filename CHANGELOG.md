# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0-alpha.2] - 2026-08-13

### Added

- Add the chip-neutral append-only `WriteStorage` capability used by format
  owners to program previously erased bytes without coupling erase or garbage
  collection policy into the storage contract.

## [0.1.0-alpha.1] - 2026-08-12

### Added

- Establish bounded chip-neutral read storage contracts and memory-mapped,
  slice-backed, and region-backed implementations.
