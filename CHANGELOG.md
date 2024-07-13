# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog][],
and this project adheres to [Semantic Versioning][].


## [v1.0.4](https://github.com/mishamyrt/runon/releases/tag/v1.0.4) - 2024-07-13
### Features
- set log level from arguments
- dim log timestamp

### Refactoring
- clean up action runner
- isolate logger
- split config and handling

### Testing
- add logger tests


## [v1.0.3](https://github.com/mishamyrt/runon/releases/tag/v1.0.3) - 2024-07-12
### Bug Fixes
- update changelog before release commit

### Features
- add installation script


## [v1.0.2](https://github.com/mishamyrt/runon/releases/tag/v1.0.2) - 2024-07-12
### Bug Fixes
- remove xtra deps


## [v1.0.1](https://github.com/mishamyrt/runon/releases/tag/v1.0.1) - 2024-07-12

## [v1.0.0](https://github.com/mishamyrt/runon/releases/tag/v1.0.0) - 2024-07-12
### Bug Fixes
- improve target handler search
- improve errors
- improve desk example
- improve logging
- reduce wake ups
- remove useless logger import
- remove extra logging
- avoid process deadlock

### CI
- use macos-latest
- use 5.10.1
- update swift version
- run tests on qa workflow
- add qa workflow

### Features
- add build-time variables
- add multiple handlers support
- add debounce group handling
- improve handler logging
- pass arguments to start
- add multiline script support
- improve argument passing
- add shell timeout support
- improve logging
- add status command
- add config related commands
- add `print` command
- add app event source
- add autostart control
- improve running state detection
- add audio devices event source
- use bash script for daemonizing

### Refactoring
- rework info generation
- remove constant type
- improve code splitting
- update path constants naming
- migrate shell commands to Shellac library
- transform process to class
- move single files to root
- rework script
- rework login item
- static logger
- improve config handling
- simplify screen source logic
- fully rename project
- fix code style problems
- rename project
- rename `Commands` to `Runner`
- simplify code structure

### Testing
- add more config cases
- add extensions tests
- add timeout config example

[keep a changelog]: https://keepachangelog.com/en/1.0.0/
[semantic versioning]: https://semver.org/spec/v2.0.0.html
[Unreleased]: https://github.com/mishamyrt/runon/compare/v1.0.4...HEAD
[v1.0.4]: https://github.com/mishamyrt/runon/compare/v1.0.3...v1.0.4
[v1.0.3]: https://github.com/mishamyrt/runon/compare/v1.0.2...v1.0.3
[v1.0.2]: https://github.com/mishamyrt/runon/compare/v1.0.1...v1.0.2
[v1.0.1]: https://github.com/mishamyrt/runon/compare/v1.0.0...v1.0.1
