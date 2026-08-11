use std::{
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn sigye(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_sigye"))
        .args(args)
        .output()
        .expect("run sigye")
}

fn line(output: &std::process::Output) -> &str {
    assert!(output.status.success(), "status: {}", output.status);
    assert!(output.stderr.is_empty(), "stderr: {:?}", output.stderr);
    assert!(!output.stdout.contains(&b'\x1b'));
    let stdout = std::str::from_utf8(&output.stdout).expect("UTF-8 stdout");
    let line = stdout.strip_suffix('\n').expect("trailing newline");
    assert!(!line.contains(['\n', '\r']), "expected exactly one line");
    line
}

fn assert_digits_except(value: &str, separators: &[(usize, u8)]) {
    for (index, byte) in value.bytes().enumerate() {
        if let Some((_, expected)) = separators.iter().find(|(position, _)| *position == index) {
            assert_eq!(byte, *expected, "separator at byte {index}");
        } else {
            assert!(byte.is_ascii_digit(), "digit at byte {index}");
        }
    }
}

#[test]
fn help_version_and_invalid_arguments_describe_shipped_binary() {
    let version = sigye(&["--version"]);
    assert!(version.status.success());
    assert_eq!(
        version.stdout,
        format!("sigye {}\n", env!("CARGO_PKG_VERSION")).as_bytes()
    );
    assert!(version.stderr.is_empty());

    let help = sigye(&["--help"]);
    assert!(help.status.success());
    let stdout = std::str::from_utf8(&help.stdout).expect("UTF-8 help");
    for expected in [
        "Usage: sigye [OPTIONS]",
        "--once",
        "--format <FORMAT>",
        "--screensaver",
    ] {
        assert!(stdout.contains(expected), "missing {expected:?}");
    }
    assert!(help.stderr.is_empty());

    let invalid = sigye(&["--not-a-sigye-option"]);
    assert!(!invalid.status.success());
    assert!(invalid.stdout.is_empty());
    assert!(
        std::str::from_utf8(&invalid.stderr)
            .expect("UTF-8 error")
            .contains("unexpected argument '--not-a-sigye-option'")
    );
}

#[test]
fn once_formats_are_scriptable() {
    let human_output = sigye(&["--once"]);
    let human = line(&human_output);
    assert_eq!(human.len(), 19);
    assert_digits_except(
        human,
        &[(4, b'-'), (7, b'-'), (10, b' '), (13, b':'), (16, b':')],
    );
    assert!(human[11..13].parse::<u8>().unwrap() <= 23);
    assert!(human[14..16].parse::<u8>().unwrap() <= 59);
    assert!(human[17..19].parse::<u8>().unwrap() <= 59);

    let before = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let unix = line(&sigye(&["--once", "--format", "unix"]))
        .parse::<i64>()
        .expect("Unix timestamp");
    let after = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    assert!((before.saturating_sub(1) as i64..=after.saturating_add(1) as i64).contains(&unix));

    let iso_output = sigye(&["--once", "--format", "iso"]);
    let iso = line(&iso_output);
    assert_eq!(iso.len(), 25);
    assert_digits_except(
        iso,
        &[
            (4, b'-'),
            (7, b'-'),
            (10, b'T'),
            (13, b':'),
            (16, b':'),
            (19, iso.as_bytes()[19]),
            (22, b':'),
        ],
    );
    assert!(matches!(iso.as_bytes()[19], b'+' | b'-'));
    assert!(iso[11..13].parse::<u8>().unwrap() <= 23);
    assert!(iso[14..16].parse::<u8>().unwrap() <= 59);
    assert!(iso[17..19].parse::<u8>().unwrap() <= 59);
    assert!(iso[20..22].parse::<u8>().unwrap() <= 23);
    assert!(iso[23..25].parse::<u8>().unwrap() <= 59);

    let hex_output = sigye(&["--once", "--format", "hex"]);
    let hex = line(&hex_output);
    assert_eq!(hex.len(), 8);
    assert_eq!(hex.as_bytes()[2], b':');
    assert_eq!(hex.as_bytes()[5], b':');
    let parts: Vec<_> = hex
        .split(':')
        .map(|part| u8::from_str_radix(part, 16).expect("hex component"))
        .collect();
    assert!(parts[0] <= 0x17);
    assert!(parts[1] <= 0x3b);
    assert!(parts[2] <= 0x3b);
}

#[cfg(unix)]
mod tui {
    use std::{
        fs,
        io::{Read, Write},
        path::PathBuf,
        sync::{
            atomic::{AtomicUsize, Ordering},
            mpsc::{self, Receiver},
        },
        thread,
        time::{Duration, Instant},
    };

    use portable_pty::{CommandBuilder, PtySize, native_pty_system};

    static NEXT_TEMP: AtomicUsize = AtomicUsize::new(0);

    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "sigye-e2e-{}-{}",
                std::process::id(),
                NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).expect("create temporary config directory");
            Self(path)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn receive_until(
        receiver: &Receiver<Vec<u8>>,
        output: &mut Vec<u8>,
        predicate: impl Fn(&str) -> bool,
    ) -> bool {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            let remaining = deadline.saturating_duration_since(Instant::now());
            match receiver.recv_timeout(remaining) {
                Ok(chunk) => output.extend(chunk),
                Err(_) => return false,
            }
            if predicate(&visible_text(output)) {
                return true;
            }
        }
        false
    }

    fn visible_text(output: &[u8]) -> String {
        let mut plain = Vec::with_capacity(output.len());
        let mut bytes = output.iter().copied().peekable();
        while let Some(byte) = bytes.next() {
            if byte != b'\x1b' {
                plain.push(byte);
                continue;
            }
            match bytes.next() {
                Some(b'[') => {
                    for byte in bytes.by_ref() {
                        if (0x40..=0x7e).contains(&byte) {
                            break;
                        }
                    }
                }
                Some(b']') => {
                    while let Some(byte) = bytes.next() {
                        if byte == 7 || (byte == b'\x1b' && bytes.next() == Some(b'\\')) {
                            break;
                        }
                    }
                }
                Some(_) | None => {}
            }
        }
        String::from_utf8_lossy(&plain).into_owned()
    }

    fn click(writer: &mut impl Write, column: u16, row: u16) {
        write!(writer, "\x1b[<0;{column};{row}M\x1b[<0;{column};{row}m").expect("send mouse click");
        writer.flush().expect("flush mouse click");
    }

    #[test]
    fn tui_mouse_controls_footer_and_restores_terminal() {
        let temp = TempDir::new();
        let temp_path = temp.0.clone();
        let pair = native_pty_system()
            .openpty(PtySize {
                rows: 40,
                cols: 120,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("open PTY");
        let mut command = CommandBuilder::new(env!("CARGO_BIN_EXE_sigye"));
        command.args([
            "--font", "Standard", "--theme", "cyan", "--bg", "none", "--mode", "clock",
        ]);
        command.env("HOME", &temp.0);
        command.env("XDG_CONFIG_HOME", &temp.0);
        command.env("TERM", "xterm-256color");

        let mut child = pair.slave.spawn_command(command).expect("spawn sigye");
        drop(pair.slave);
        let mut reader = pair.master.try_clone_reader().expect("clone PTY reader");
        let mut writer = pair.master.take_writer().expect("take PTY writer");
        let (sender, receiver) = mpsc::channel();
        let reader_thread = thread::spawn(move || {
            let mut buffer = [0; 4096];
            while let Ok(count) = reader.read(&mut buffer) {
                if count == 0 {
                    break;
                }
                if sender.send(buffer[..count].to_vec()).is_err() {
                    break;
                }
            }
        });

        let mut output = Vec::new();
        let rendered = receive_until(&receiver, &mut output, |text| text.contains("[s]settings"));
        if !rendered {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "TUI did not render within five seconds; output: {:?}",
                visible_text(&output)
            );
        }

        // Footer positions are literal 1-based terminal coordinates at 120x40.
        click(&mut writer, 88, 40);
        let settings_rendered =
            receive_until(&receiver, &mut output, |text| text.contains("[Enter] save"));
        if !settings_rendered {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "mouse click did not open settings within five seconds; output: {:?}",
                visible_text(&output)
            );
        }

        writer.write_all(b"\x1b").expect("close settings");
        writer.flush().expect("flush close key");
        match receiver.recv_timeout(Duration::from_secs(5)) {
            Ok(chunk) => output.extend(chunk),
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("settings did not close within five seconds");
            }
        }

        click(&mut writer, 102, 40);
        let help_rendered = receive_until(&receiver, &mut output, |text| text.contains("Controls"));
        if !help_rendered {
            let _ = child.kill();
            let _ = child.wait();
            panic!("mouse click did not open help within five seconds");
        }

        click(&mut writer, 1, 1);
        click(&mut writer, 112, 40);
        drop(writer);
        let deadline = Instant::now() + Duration::from_secs(5);
        let status = loop {
            if let Some(status) = child.try_wait().expect("poll sigye") {
                break status;
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("sigye did not exit within five seconds");
            }
            thread::sleep(Duration::from_millis(10));
        };
        while let Ok(chunk) = receiver.recv_timeout(Duration::from_millis(50)) {
            output.extend(chunk);
        }
        reader_thread.join().expect("join PTY reader");

        let enter = output
            .windows(8)
            .position(|bytes| bytes == b"\x1b[?1049h")
            .expect("enter alternate screen");
        let mouse_on = output
            .windows(8)
            .position(|bytes| bytes == b"\x1b[?1000h")
            .expect("enable mouse capture");
        assert!(!output.windows(8).any(|bytes| bytes == b"\x1b[?1002h"));
        assert!(!output.windows(8).any(|bytes| bytes == b"\x1b[?1003h"));
        assert!(!output.windows(8).any(|bytes| bytes == b"\x1b[?1015h"));
        assert!(output.windows(8).any(|bytes| bytes == b"\x1b[?1006h"));
        let render = output
            .windows(b"[s]".len())
            .position(|bytes| bytes == b"[s]")
            .expect("render content");
        let mouse_off = output
            .windows(8)
            .position(|bytes| bytes == b"\x1b[?1000l")
            .expect("disable mouse capture");
        let leave = output
            .windows(8)
            .position(|bytes| bytes == b"\x1b[?1049l")
            .expect("leave alternate screen");
        assert!(enter < mouse_on && mouse_on < render);
        assert!(render < mouse_off && mouse_off < leave);
        assert!(status.success(), "exit status: {}", status.exit_code());

        drop(temp);
        assert!(
            !temp_path.exists(),
            "temporary config directory was not removed"
        );
    }
}
