# Minimal Clock View

## Goal

Resolve GitHub issue #49 with a persisted Clock-mode toggle that shows only the centered ASCII clock.

## Behavior

- `z` toggles minimal view while Clock mode is active.
- The toggle is saved immediately and restored on the next launch.
- Minimal view keeps the selected font, time format, seconds setting, colors, animations, colon blinking, and animated background.
- Minimal view hides the date, sunrise and sunset details, display-format label, ISO detail, clipboard toast, day and year progress bars, and key hints.
- Existing keyboard controls remain active while the hints are hidden, including `z` to leave minimal view.
- Other display modes are unchanged.

## Implementation

Add a serde-defaulted `minimal_mode` boolean to `Config`, defaulting to `false` so existing configuration files remain compatible. Clock rendering reads that value directly. When it is enabled, Clock mode uses a three-part vertical layout—flexible space, font height, flexible space—and skips all secondary rendering. The existing clock rendering, background pass, and input dispatch remain unchanged.

Clock mode handles `z` by flipping the config value and calling the existing config save path. A save failure follows the application's existing behavior and prints a warning without discarding the in-memory toggle.

## Testing

- A render regression test enables minimal view and verifies that the ASCII clock remains while the date, progress bars, and key hints are absent.
- Config tests verify that missing `minimal_mode` values default to `false` and that an enabled value serializes and deserializes.
- The workspace test, format, and lint commands verify the complete change.

## Documentation

Update the README feature list, CLI/config example where appropriate, and Clock-mode keybinding table to describe the saved `z` toggle.
