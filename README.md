# sigye (시계)

[![Crates.io](https://img.shields.io/crates/v/sigye)](https://crates.io/crates/sigye)
[![License](https://img.shields.io/crates/l/sigye)](https://github.com/am2rican5/sigye/blob/main/LICENSE)
[![Built With Ratatui](https://img.shields.io/badge/Built_With_Ratatui-000?logo=ratatui&logoColor=fff)](https://ratatui.rs/)

A feature-rich terminal clock with ASCII art fonts, animated backgrounds, and productivity timers. The name "sigye" (시계) means "clock" in Korean.

![sigye demo](assets/demo.gif)

## Features

- **6 display modes** — Clock, Pomodoro, Timer, Stopwatch, World Clock, and Countdown
- **Developer clock formats** — Unix timestamp, ISO 8601, and hex time display (cycle with `f`)
- **Clipboard hotkeys** — Copy unix timestamp (`u`) or ISO 8601 (`i`) to clipboard instantly
- **Day & year progress bars** — Ambient progress indicators at a glance
- **Scriptable output** — `sigye --once --format unix` for shell pipelines
- **40+ bundled FIGlet fonts** — From classic Standard to stylish Star Wars
- **18 color themes** — 7 static colors and 11 gradient palettes
- **5 animation styles** — None, Shifting, Pulsing, Wave, and Reactive
- **21 animated backgrounds** — Starfield, matrix rain, weather effects, cherry blossoms, twilight, and system-reactive visuals
- **Desktop notifications** — Alerts for Pomodoro phase changes and timer completion
- **Live weather backgrounds** — Auto-selects rain, snow, fog, or sun based on real conditions via wttr.in
- **System-reactive backgrounds** — CPU, memory, and network metrics drive visual effects
- **World Clock** — Display multiple timezones simultaneously
- **CLI arguments** — Launch directly into any mode, theme, font, or background
- **Screensaver and demo modes** — Fullscreen ambient display or auto-cycling showcase
- **12/24 hour format** — Toggle with a single keypress
- **Live settings preview** — See changes in real time before saving
- **Persistent configuration** — Settings saved automatically to TOML
- **Custom font support** — Drop FIGlet `.flf` files into `~/.config/sigye/fonts/`

## How sigye Compares

| Feature | sigye | tty-clock | peaclock | clock-tui | timr-tui |
|---------|-------|-----------|----------|-----------|----------|
| Maintained | Active | 2021 | 2020 | Active | Active |
| Language | Rust | C | C++ | Rust | Rust |
| Clock | Yes | Yes | Yes | Yes | Yes |
| Pomodoro | Yes | -- | -- | -- | Yes |
| Timer | Yes | -- | Yes | Yes | Yes |
| Stopwatch | Yes | -- | Yes | -- | Yes |
| World Clock | Yes | -- | -- | -- | -- |
| Life Countdown | Yes | -- | -- | -- | -- |
| Unix/ISO/Hex Time | Yes | -- | -- | -- | -- |
| Clipboard Copy | Yes | -- | -- | -- | -- |
| Scriptable Output | Yes | -- | -- | -- | -- |
| FIGlet Fonts | 40+ | 1 | 3 | 1 | 7 |
| Color Themes | 18 | 8 | 256 | Basic | Basic |
| Animated Backgrounds | 21 | -- | -- | -- | -- |
| Live Weather | Yes | -- | -- | -- | -- |
| System-Reactive | Yes | -- | -- | -- | -- |
| Desktop Notifications | Yes | -- | -- | -- | Yes* |
| Screensaver Mode | Yes | Yes | -- | -- | -- |
| Config File | TOML | -- | Yes | -- | Yes |
| Cross-Platform | Yes | Linux | Linux | Yes | Yes |

## Installation

### Homebrew (macOS / Linux)

```bash
brew install am2rican5/sigye/sigye
```

Installs a prebuilt binary — no Rust toolchain required.

### Pre-built binaries

Download pre-built binaries from the [GitHub Releases](https://github.com/am2rican5/sigye/releases) page.

Supported platforms: Linux x86_64 gnu/musl, Linux aarch64, macOS Intel + Apple Silicon, Windows x86_64.

### From crates.io

```bash
cargo install sigye
```

### From source

```bash
git clone https://github.com/am2rican5/sigye
cd sigye
cargo install --path crates/sigye
```

## Usage

```bash
sigye
```

Press `s` to open the settings dialog, or use the keybindings below to adjust on the fly.

## CLI Options

```
sigye [OPTIONS]

Options:
  --screensaver          Fullscreen ambient mode (no UI chrome)
  --demo                 Auto-cycle themes, backgrounds, and fonts
  --font <NAME>          Set font (e.g., "Standard", "Banner", "Doom")
  --theme <NAME>         Set color theme (e.g., "neon", "fire", "aurora")
  --bg <NAME>            Set background (e.g., "matrix", "aurora", "weather")
  --mode <MODE>          Set display mode (clock, pomodoro, timer, stopwatch, worldclock, countdown)
  --tz <LABEL=TZ>        Add world clock timezone (repeatable)
  --once                 Print time once and exit (no TUI)
  --format <FORMAT>      Output format for --once (human, unix, iso, hex) [default: human]
  -h, --help             Print help
  -V, --version          Print version
```

### Examples

```bash
# Scripting & developer use
sigye --once                                    # Print current time and exit
sigye --once --format unix                      # Print unix timestamp (e.g., 1743494400)
sigye --once --format iso                       # Print ISO 8601 (e.g., 2026-04-01T14:30:00+09:00)
sigye --once --format hex                       # Print hex time (e.g., 0E:1E:2D)

# TUI modes
sigye --screensaver --bg aurora --theme neon
sigye --demo
sigye --mode worldclock --tz "Seoul=Asia/Seoul" --tz "Berlin=Europe/Berlin"
sigye --mode pomodoro --font Doom --theme fire
```

## Keybindings

### Global

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit |
| `m` | Cycle display mode (Clock / Pomodoro / Timer / Stopwatch / World Clock / Countdown) |
| `M` | Open mode picker dialog (1-6 to jump, ↑↓ to navigate, Enter to pick) |
| `t` | Toggle 12/24 hour format |
| `c` | Cycle color theme |
| `a` | Cycle animation style |
| `b` | Cycle background style |
| `s` | Open settings dialog |
| `?` | Show help overlay |

### Clock Mode

| Key | Action |
|-----|--------|
| `f` | Cycle display format (Clock → Unix Timestamp → ISO 8601 → Hex Time) |
| `u` | Copy unix timestamp to clipboard |
| `i` | Copy ISO 8601 timestamp to clipboard |

### Pomodoro Mode

| Key | Action |
|-----|--------|
| `Space` | Pause / resume |
| `r` | Reset current phase |
| `n` | Skip to next phase |

### Timer Mode

| Key | Action |
|-----|--------|
| `Space` | Pause / resume |
| `r` | Reset timer |
| `+` / `=` | Add time |
| `-` | Subtract time |

### Stopwatch Mode

| Key | Action |
|-----|--------|
| `Space` | Pause / resume |
| `r` | Reset stopwatch |
| `l` | Record lap |

### Countdown Mode

| Key | Action |
|-----|--------|
| `n` / `→` | Next event |
| `p` / `←` | Previous event |
| `e` | Open event management dialog (add / edit / delete) |

### Settings Dialog

| Key | Action |
|-----|--------|
| `Up` / `k` | Previous field |
| `Down` / `j` | Next field |
| `Left` / `h` | Previous value |
| `Right` / `l` | Next value |
| `Enter` | Save settings |
| `Esc` | Cancel |

## Configuration

Settings are stored at `~/.config/sigye/config.toml`:

```toml
font_name = "Standard"
color_theme = "Cyan"
time_format = "TwentyFourHour"
animation_style = "None"
animation_speed = "Medium"
colon_blink = false
show_seconds = true
background_style = "None"
weather_location = ""          # Empty for auto-detect, or city name (e.g., "Seoul")
pomodoro_work_mins = 25
pomodoro_break_mins = 5
pomodoro_long_break_mins = 15
pomodoro_sessions_until_long = 4
pomodoro_sound = true
```

### World Clock

Add timezones to display in World Clock mode:

```toml
world_clock_zones = [
    "New York=America/New_York",
    "London=Europe/London",
    "Tokyo=Asia/Tokyo",
    "Seoul=Asia/Seoul",
]
```

### Countdown Events

Add life events to display in Countdown mode. `target` accepts an ISO 8601
date (`YYYY-MM-DD`, interpreted as local midnight), a naive local datetime
(`YYYY-MM-DDTHH:MM:SS`), or a full RFC 3339 datetime with offset. Set
`since = true` to count **up** from the target instead of down to it:

```toml
[[countdown_events]]
name = "Launch"
target = "2026-06-01T09:00:00+09:00"

[[countdown_events]]
name = "Birthday"
target = "2026-09-15"

[[countdown_events]]
name = "Sober"
target = "2023-02-14"
since = true
```

In Countdown mode, press `n` / `p` (or arrow keys) to cycle between events.
Events far from now show a day count; events under 24 hours away switch to
`HH:MM:SS` for tangible urgency. A date-only event on today's date surfaces
as `TODAY`.

### Custom Fonts

Place FIGlet font files (`.flf`) in `~/.config/sigye/fonts/` and they will appear in the settings dialog.

## Color Themes

### Static Colors

Cyan, Green, White, Magenta, Yellow, Red, Blue

### Dynamic Gradients

| Theme | Description |
|-------|-------------|
| Rainbow | Horizontal rainbow spectrum |
| Rainbow V | Vertical rainbow |
| Warm | Red to Orange to Yellow |
| Cool | Blue to Cyan to Green |
| Ocean | Dark Blue to Cyan to Teal |
| Neon | Magenta to Cyan (synthwave) |
| Fire | Dark Red to Orange to Yellow |
| Frost | White to Ice Blue to Steel Blue |
| Aurora | Green to Cyan to Blue to Purple |
| Winter | Deep Blue to Royal Blue to Ice Blue |
| Sakura | Sakura Pink to Lavender Blush |

## Background Styles

### Classic

| Style | Description |
|-------|-------------|
| Starfield | Twinkling stars with varying brightness |
| Matrix | Falling green Matrix-style characters |
| Gradient | Flowing diagonal color wave |

### Weather & Atmospheric

| Style | Description |
|-------|-------------|
| Weather | Auto-selects based on real-time conditions via wttr.in |
| Sunny | Radiant sun with animated rays |
| Cloudy | Layered drifting clouds |
| Foggy | Ground-hugging mist effect |
| Rainy | Falling rain droplets |
| Stormy | Rain with lightning flashes |
| Windy | Horizontal wind streaks |
| Snowfall | Drifting snowflakes in shades of blue |
| Frost | Ice crystals growing from screen edges |
| Aurora | Northern lights in green, cyan, blue, and purple |

### Twilight & Spring

| Style | Description |
|-------|-------------|
| Dawn | Sunrise gradient with fading stars |
| Dusk | Sunset gradient with emerging stars |
| Sakura | Cherry blossom petals drifting down |

### System-Reactive

Driven by real-time system metrics (CPU, memory, network):

| Style | Description |
|-------|-------------|
| Sys Pulse | CPU usage drives pulsing rings from center |
| Resource | Memory usage controls wave amplitude |
| Data Flow | Network I/O drives particle density |
| Heat Map | Combined metrics as color intensity |

## Bundled Fonts

3D-ASCII, Acrobatic, Alligator, Alphabet, ANSI Regular, ANSI Shadow, Avatar, Banner, Bell, Big, Big Money-ne, Block, BlurVision ASCII, Chunky, Colossal, Doh, Doom, Electronic, Epic, Graffiti, Ivrit, Larry 3D, Lean, Mini, Mono 9, Mono 12, Ogre, Poison, Puffy, Rebel, Rectangles, Script, Shadow, Slant, Small, Speed, Standard, Star Wars, Terrace, Tmplr

## Contributing

```bash
git clone https://github.com/am2rican5/sigye
cd sigye
cargo build
cargo test --workspace
cargo fmt -- --check
cargo clippy
```

### Workspace conventions

- **MSRV is 1.97.1**, declared in `[workspace.package]` and inherited by every crate.
- **All dependencies live in `[workspace.dependencies]`** in the root `Cargo.toml`, and
  crates pull them in with `foo.workspace = true`. Version requirements are written as
  `major.minor`.
- **No `unsafe`.** The workspace sets `unsafe_code = "forbid"`, so `unsafe` is a compile
  error in `crates/` and cannot be re-enabled by a local `#[allow]`.
- **No `mod.rs`.** Modules use the self-named layout (`modes.rs` alongside `modes/`).
  This is now enforced by `clippy::mod_module_files`.
- **Releases bump versions in the root `Cargo.toml` only.** Both `[workspace.package]
  version` and the four internal `sigye-* = { version = ... }` entries have to move
  together.

## License

Copyright (c) am2rican5

This project is licensed under the MIT license ([LICENSE](./LICENSE) or <http://opensource.org/licenses/MIT>)
