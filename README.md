<p align="center">
    <img src="./assets/logo.svg" alt="RunOn logo" />
</p>

[![Quality Assurance](https://github.com/mishamyrt/runon/actions/workflows/qa.yaml/badge.svg)](https://github.com/mishamyrt/runon/actions/workflows/qa.yaml) [![Version](https://img.shields.io/github/v/tag/mishamyrt/runon?sort=semver)](https://github.com/mishamyrt/nuga-lib/tags)

RunOn is a utility for running commands on macOS system events.

## Installation

### Script

The easiest way is to use an installation script. To install using the script, run the command:

```sh
curl -skSfL https://raw.githubusercontent.com/mishamyrt/runon/main/scripts/install.sh | bash
```

This command will download and run the [installation script](./scripts/install.sh).

### Build from source

Building the project is currently only possible on macOS. Swift 5.10 is required for the build.

```sh
# Build the project from source code
make build
# Install runon to the system
sudo make install
```

## Configuration

To print the configuration file path, run the command:

```sh
runon config-path
```

The configuration is described in the format:

> When `SOURCE` emits an `EVENT` [with `DATA`] execute a `COMMAND`

Example:

```yaml
actions:
  # If my work monitor is connected, turn on the desk backlight
  - on: screen:connected
    with: Mi 27 NU
    run: myrt_desk on# If the monitor is disconnected, turn off the desk backlight
  - on: screen:disconnected
    with: Mi 27 NU
    run: myrt_desk off
```

If you want to run a command on an event, regardless of the input (`with`), then use the simplified notation:

```yaml
actions:
  # If any monitor is disconnected, set brightness to 60%
  - on: screen:disconnected
    run: lunar set 60
```

Execution of commands is time-limited. The default maximum time is 30 seconds. It can be set for each command separately.

```yaml
actions:
  # the process will be terminated in 20 seconds
  - on: screen:disconnected
    run: sleep 30
    timeout: 20s
```

To execute multiple commands sequentially, describe them in a multiline string, as in the script:

```yaml
actions:
  - on: screen:connected
    run: |
      setup_audio
      desk_lights on
```

Multiple sources or target events can be specified in a similar way:

```yaml
actions:
  - on: |
      screen:connected
      screen:disconnected
    with: |
      Mi 27 NU
      ROG 32U
    run: echo 'display changed'
```

### Groups

Several actions can be combined into groups. Within a group, you can set a minimum interval between two actions.

```yaml
actions:
  - on: screen:connected
    with: Mi 27 NU
    run: myrt_desk on
    group: desk
  - on: screen:disconnected
    with: Mi 27 NU
    run: myrt_desk off
    group: desk

groups:
  # avoid flickering
  - name: desk
    debounce: 5s
```

## Event sources

The following event sources can be subscribed to:

- `screen` — connected displays list change. declares `connected`, `disconnected`, `locked` and `unlocked` events. In `with` takes the display name.
- `audio` — connected audio device list change. declares `connected` and `disconnected` events. In `with` takes the audio device name. Handles both input and output devices.
- `app` — active application change. declares `activated` and `deactivated` events. In `with` takes the app bundle identifier (like `com.microsoft.VSCode`).

## Usage

### Service

The background service can be started and stopped with the `runon` command.

```sh
runon start # starts the service
runon stop # stops the service
```
