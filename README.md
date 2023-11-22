<p align="center">
    <img src="./assets/logo.svg" alt="Run if" />
</p>

[![Quality Assurance](https://github.com/mishamyrt/runif/actions/workflows/qa.yaml/badge.svg)](https://github.com/mishamyrt/runif/actions/workflows/qa.yaml)

RunIf is a utility for running commands on macOS system events.

## Configuration

The configuration is described in the format:

> If the `SOURCE` has emitted an `EVENT` [with `DATA`], then execute the `COMMAND`.

Example:

```yaml
# If my work monitor is connected, turn on the desk backlight
- if: screen:connected
  with: Mi 27 NU
  run: myrt_desk on
# If the monitor is disconnected, turn off the desk backlight
- if: screen:disconnected
  with: Mi 27 NU
  run: myrt_desk off
```

If you want to run a command on an event, regardless of the input (`with`), then use the simplified notation:

```yaml
# If any monitor is disconnected, set brightness to 60%
- if: screen:disconnected
  run: lunar set 60
```

## Event sources

The following event sources can be subscribed to:

- `screen` — connected displays list change. declares `connected` and `disconnected` events. In `with` takes the display name.
- `audio`  — connected audio device list change. declares `connected` and `disconnected` events. In `with` takes the audio device name. Handles both input and output devices.

## Usage

The application is controlled through commands:

- `start` — starts the application in the background.
- `stop` — stops the background application.
- `restart` — restarts background application.
- `run` — runs the application in the current process without daemonization.
- `autostart` — controls the automatic start-up of the application.
