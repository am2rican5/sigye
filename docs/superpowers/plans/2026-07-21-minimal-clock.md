# Minimal Clock View Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a saved `z` toggle in Clock mode that renders only the centered ASCII clock.

**Architecture:** Store one serde-defaulted boolean in the existing `Config`. Clock mode reads it directly, uses a three-row vertical layout when enabled, skips secondary chrome after drawing the clock, and persists changes through the existing `Config::save` path.

**Tech Stack:** Rust, serde, TOML, Ratatui `TestBackend`, Cargo

## Global Constraints

- Keep backgrounds, colors, animations, time format, seconds, and colon blinking unchanged.
- Hide date, sunrise/sunset, format details, clipboard toast, progress bars, and key hints only in Clock mode.
- Keep all keyboard controls active while minimal view is enabled.
- Add no dependencies or generic UI-chrome abstraction.

---

### Task 1: Persist the minimal-view setting

**Files:**
- Modify: `crates/sigye-config/src/lib.rs:11-199`
- Test: `crates/sigye-config/src/lib.rs`

**Interfaces:**
- Consumes: serde's existing `Serialize` and `Deserialize` derives and the crate's TOML dependency.
- Produces: `Config::minimal_mode: bool`, defaulting to `false` when absent.

- [ ] **Step 1: Write failing config compatibility tests**

Append this test module to `crates/sigye-config/src/lib.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_minimal_mode_defaults_to_false() {
        let config: Config = toml::from_str("").unwrap();

        assert!(!config.minimal_mode);
    }

    #[test]
    fn minimal_mode_round_trips() {
        let mut config = Config::default();
        config.minimal_mode = true;

        let encoded = toml::to_string(&config).unwrap();
        let decoded: Config = toml::from_str(&encoded).unwrap();

        assert!(decoded.minimal_mode);
    }
}
```

- [ ] **Step 2: Run the focused tests and verify red**

Run: `cargo test -p sigye-config minimal_mode`

Expected: compilation fails because `Config` has no `minimal_mode` field.

- [ ] **Step 3: Add the minimal config field**

Add this field immediately after `show_seconds` in `Config`:

```rust
    /// Whether Clock mode hides all secondary UI chrome.
    #[serde(default)]
    pub minimal_mode: bool,
```

Add this initializer immediately after `show_seconds` in `Config::default()`:

```rust
            minimal_mode: false,
```

- [ ] **Step 4: Run the focused tests and verify green**

Run: `cargo test -p sigye-config minimal_mode`

Expected: 2 tests pass, 0 fail.

- [ ] **Step 5: Commit the config change**

```bash
git add crates/sigye-config/src/lib.rs
git commit -m "feat: persist minimal clock preference"
```

---

### Task 2: Toggle and render the minimal Clock view

**Files:**
- Modify: `crates/sigye/src/modes/clock.rs:216-420`
- Test: `crates/sigye/src/modes/clock.rs:428-507`

**Interfaces:**
- Consumes: `RenderContext::config`, `Config::minimal_mode`, and `Config::save() -> Result<(), ConfigError>`.
- Produces: `z` handling in `ClockMode::handle_key` and minimal rendering in `ClockMode::render`.

- [ ] **Step 1: Write the failing render regression test**

Add this test beside the existing Clock render tests:

```rust
    #[test]
    fn render_minimal_view_shows_only_centered_clock() {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
        let mode = ClockMode::new();
        let mut ctx = render_context(false);
        ctx.config.minimal_mode = true;
        ctx.sunrise_sunset = Some(("06:00".into(), "18:00".into()));
        let date = Local::now().format("%A, %B %-d, %Y").to_string();

        terminal.draw(|frame| mode.render(frame, &ctx)).unwrap();

        let backend = terminal.backend();
        assert!(!buffer_contains_text(backend, &date));
        assert!(!buffer_contains_text(backend, "Sunrise"));
        assert!(!buffer_contains_text(backend, "Day"));
        assert!(!buffer_contains_text(backend, "Year"));
        assert!(!buffer_contains_text(backend, "[s] settings"));

        let rows = occupied_rows(backend);
        assert!(!rows.is_empty());
        let top_padding = rows[0];
        let bottom_padding = 39 - rows[rows.len() - 1];
        assert!(top_padding.abs_diff(bottom_padding) <= 1);
    }
```

Add this helper beside `buffer_contains_text`:

```rust
    fn occupied_rows(backend: &TestBackend) -> Vec<u16> {
        let buffer = backend.buffer();
        let area = buffer.area;
        (area.top()..area.bottom())
            .filter(|&y| {
                (area.left()..area.right()).any(|x| {
                    buffer
                        .cell((x, y))
                        .is_some_and(|cell| cell.symbol() != " ")
                })
            })
            .collect()
    }
```

- [ ] **Step 2: Run the focused test and verify red**

Run: `cargo test -p sigye render_minimal_view_shows_only_centered_clock`

Expected: test fails because the date, progress bars, and key hints still render.

- [ ] **Step 3: Use a centered layout and skip Clock chrome**

Replace the current `chunks` construction at the start of `ClockMode::render` with:

```rust
        let chunks = if ctx.config.minimal_mode {
            Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(font_height),
                Constraint::Fill(1),
            ])
            .split(area)
        } else {
            let sun_height = if ctx.sunrise_sunset.is_some() { 1 } else { 0 };
            Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(font_height),
                Constraint::Length(2),
                Constraint::Length(1),
                Constraint::Length(sun_height),
                Constraint::Length(1),
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(area)
        };
```

Immediately after the existing `render_ascii_text` call, add:

```rust
        if ctx.config.minimal_mode {
            return;
        }
```

This retains the existing clock text calculation and ASCII renderer while preventing every secondary Clock element from rendering.

- [ ] **Step 4: Add the saved `z` toggle**

Add this first arm to `ClockMode::handle_key`:

```rust
            KeyCode::Char('z') => {
                ctx.config.minimal_mode = !ctx.config.minimal_mode;
                if let Err(e) = ctx.config.save() {
                    eprintln!("Warning: Failed to save config: {e}");
                }
                true
            }
```

Add this first pair to `ClockMode::key_hints` so the control is discoverable before chrome is hidden:

```rust
            ("z", "minimal"),
```

- [ ] **Step 5: Run Clock tests and verify green**

Run: `cargo test -p sigye modes::clock::tests`

Expected: all Clock tests pass, including the new minimal render test.

- [ ] **Step 6: Commit the Clock behavior**

```bash
git add crates/sigye/src/modes/clock.rs
git commit -m "feat: add minimal clock view"
```

---

### Task 3: Document and verify the feature

**Files:**
- Modify: `README.md:11-27`
- Modify: `README.md:143-151`
- Modify: `README.md:192-208`

**Interfaces:**
- Consumes: the saved `minimal_mode` config field and `z` Clock binding from Tasks 1 and 2.
- Produces: user-facing discovery and configuration documentation.

- [ ] **Step 1: Document the feature, key, and config value**

Add this feature bullet after the progress-bar bullet:

```markdown
- **Minimal clock view** — Press `z` to hide all secondary UI and center the clock; the preference is saved
```

Add this row to the Clock Mode keybinding table:

```markdown
| `z` | Toggle saved minimal clock view |
```

Add this line after `show_seconds` in the configuration example:

```toml
minimal_mode = false
```

- [ ] **Step 2: Run full verification**

Run these commands and require each to exit 0:

```bash
cargo test --workspace
cargo fmt -- --check
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
```

Expected: all workspace tests pass, formatting is unchanged, Clippy reports no warnings, and Git reports no whitespace errors.

- [ ] **Step 3: Commit the documentation**

```bash
git add README.md
git commit -m "docs: describe minimal clock view"
```
