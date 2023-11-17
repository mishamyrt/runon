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

## Event providers

The following event sources can be subscribed to:

* `screen` — change connected displays. declares `connected` and `disconnected` events. In `with` takes the display name.


