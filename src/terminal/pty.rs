use std::io::{Read, Write};
use std::thread;

use anyhow::Result;
use crossbeam_channel::{bounded, unbounded, Receiver, Sender};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};

pub struct PtySession {
    pub bytes_rx: Receiver<Vec<u8>>,
    pub input_tx: Sender<Vec<u8>>,
    pub resize_tx: Sender<(u16, u16)>,
    _child: Box<dyn portable_pty::Child + Send + Sync>,
}

pub fn spawn_session(
    cols: u16,
    rows: u16,
    startup: Option<&crate::office::config::StartupCommand>,
) -> Result<PtySession> {
    let pty_system = NativePtySystem::default();
    let pair = pty_system.openpty(PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    let configured_shell = startup
        .and_then(|s| s.shell.clone())
        .filter(|s| !s.is_empty());
    let extra_args: Vec<String> = startup
        .map(|s| s.args.clone())
        .unwrap_or_default();

    let mut cmd = if cfg!(windows) {
        let mut c = CommandBuilder::new(configured_shell.as_deref().unwrap_or("cmd.exe"));
        for a in &extra_args {
            c.arg(a);
        }
        c
    } else {
        let shell = configured_shell.unwrap_or_else(|| {
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
        });
        let mut c = CommandBuilder::new(&shell);
        if extra_args.is_empty()
            && (shell.ends_with("bash") || shell.ends_with("zsh") || shell.ends_with("sh"))
        {
            c.arg("-i");
        }
        for a in &extra_args {
            c.arg(a);
        }
        c
    };
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");
    for var in ["HOME", "PATH", "USER", "LOGNAME", "SHELL", "LANG", "LC_ALL"] {
        if let Ok(v) = std::env::var(var) {
            cmd.env(var, v);
        }
    }
    if let Some(s) = startup {
        for (k, v) in &s.env {
            cmd.env(k, v);
        }
    }
    let cwd = startup
        .and_then(|s| s.cwd.clone())
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("HOME").ok());
    if let Some(c) = cwd {
        cmd.cwd(c);
    }

    let child = pair.slave.spawn_command(cmd)?;
    drop(pair.slave);
    eprintln!("[clankertopia] spawned shell (cols={cols}, rows={rows})");

    let (bytes_tx, bytes_rx) = bounded::<Vec<u8>>(256);
    let (input_tx, input_rx) = unbounded::<Vec<u8>>();
    let (resize_tx, resize_rx) = unbounded::<(u16, u16)>();

    let mut reader = pair.master.try_clone_reader()?;
    thread::Builder::new()
        .name("pty-reader".into())
        .spawn(move || {
            let mut buf = [0u8; 8192];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => {
                        eprintln!("[clankertopia] pty reader: EOF");
                        break;
                    }
                    Ok(n) => {
                        if bytes_tx.send(buf[..n].to_vec()).is_err() {
                            eprintln!("[clankertopia] pty reader: channel closed");
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("[clankertopia] pty reader error: {e}");
                        break;
                    }
                }
            }
        })?;

    let mut writer = pair.master.take_writer()?;
    let master = pair.master;
    thread::Builder::new()
        .name("pty-writer".into())
        .spawn(move || loop {
            crossbeam_channel::select! {
                recv(input_rx) -> msg => match msg {
                    Ok(bytes) => {
                        if writer.write_all(&bytes).is_err() {
                            break;
                        }
                        let _ = writer.flush();
                    }
                    Err(_) => break,
                },
                recv(resize_rx) -> msg => match msg {
                    Ok((c, r)) => {
                        let _ = master.resize(PtySize {
                            cols: c,
                            rows: r,
                            pixel_width: 0,
                            pixel_height: 0,
                        });
                    }
                    Err(_) => break,
                },
            }
        })?;

    Ok(PtySession {
        bytes_rx,
        input_tx,
        resize_tx,
        _child: child,
    })
}
