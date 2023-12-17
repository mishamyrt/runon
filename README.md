<p align="center">
    <img src="./assets/logo.svg" alt="Run if" />
</p>

[![Quality Assurance](https://github.com/mishamyrt/runon/actions/workflows/qa.yaml/badge.svg)](https://github.com/mishamyrt/runon/actions/workflows/qa.yaml)

RunOn is a utility for running commands on macOS system events.

## Installation

### Build from source

Building the project is currently only possible on macOS. Swift 5.9 is required for the build. 

```sh
# Build the project from source code
make build
# Install runon to the system
sudo make install
```

## Configuration

To open editing of the configuration file, run the command:

```sh
runon config
```

The configuration is described in the format:

> When `SOURCE` emits an `EVENT` [with `DATA`] execute a `COMMAND`

Example:

```yaml
# If my work monitor is connected, turn on the desk backlight
- on: screen:connected
  with: Mi 27 NU
  run: myrt_desk on
# If the monitor is disconnected, turn off the desk backlight
- on: screen:disconnected
  with: Mi 27 NU
  run: myrt_desk off
```

If you want to run a command on an event, regardless of the input (`with`), then use the simplified notation:

```yaml
# If any monitor is disconnected, set brightness to 60%
- on: screen:disconnected
  run: lunar set 60
```

Execution of commands is time-limited. The default maximum time is 30 seconds. It can be set for each command separately.

```yaml
# the process will be terminated in 20 seconds
- on: screen:disconnected
  run: sleep 30
  timeout: 20s
```

To execute multiple commands sequentially, describe them in a multiline string, as in the script:

```yaml
- on: screen:connected
  run: |
    setup_audio
    desk_lights on
```

## Event sources

The following event sources can be subscribed to:

- `screen` — connected displays list change. declares `connected` and `disconnected` events. In `with` takes the display name.
- `audio` — connected audio device list change. declares `connected` and `disconnected` events. In `with` takes the audio device name. Handles both input and output devices.
- `app` — active application change. declares `activated` and `deactivated` events. In `with` takes the app bundle identifier (like `com.microsoft.VSCode`).

## Usage

The application is controlled through commands:

- `run` — Runs the application in the current process without daemonization..
- `print` — Starts the observer in a special mode that prints all supported events.
- `autostart` — Enables, disables, or prints the autostart status.
- `start` — Starts the application in the background.
- `stop` — Stops the background application.
- `restart` — Restarts background application.
- `status` — Prints the status of the application.
- `log` — Starts the application log viewer in follow mode.
- `config` — Opens editing of the configuration file.
- `config-path` — Prints the absolute path of the configuration file.
