//! Minimal russh interactive shell host for the demo.
//!
//! Lives in the example crate — arkit_terminal has no SSH dependency.

use std::sync::Arc;

use tokio::sync::mpsc;

/// Commands from the UI thread → SSH task.
pub enum SshCmd {
    Data(Vec<u8>),
    Resize { cols: u32, rows: u32 },
    Close,
}

/// Events from SSH task → UI thread (feed into `TerminalController::feed_vt`).
#[derive(Debug, Clone)]
pub enum SshEvent {
    Connected,
    Output(Vec<u8>),
    Status(String),
    Closed(String),
}

#[derive(Clone)]
pub struct SshConnect {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub cols: u16,
    pub rows: u16,
}

/// Spawn SSH on the arkit Tokio runtime. Returns the command sender.
pub fn spawn_ssh(
    cfg: SshConnect,
    events: mpsc::UnboundedSender<SshEvent>,
) -> mpsc::UnboundedSender<SshCmd> {
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
    let _ = events.send(SshEvent::Status(format!(
        "ssh {}:{} connecting…",
        cfg.host, cfg.port
    )));
    let handle = arkit_runtime::tokio_handle();
    handle.spawn(async move {
        match run_session(cfg, cmd_rx, events.clone()).await {
            Ok(()) => {
                let _ = events.send(SshEvent::Closed("ssh closed".into()));
            }
            Err(e) => {
                let _ = events.send(SshEvent::Closed(format!("ssh error: {e}")));
            }
        }
    });
    cmd_tx
}

async fn run_session(
    cfg: SshConnect,
    mut cmd_rx: mpsc::UnboundedReceiver<SshCmd>,
    events: mpsc::UnboundedSender<SshEvent>,
) -> Result<(), String> {
    use russh::client::{self, Handler};
    use russh::{ChannelMsg, Disconnect};

    struct AcceptAll;
    impl Handler for AcceptAll {
        type Error = russh::Error;

        async fn check_server_key(
            &mut self,
            _server_public_key: &russh::keys::ssh_key::PublicKey,
        ) -> Result<bool, Self::Error> {
            Ok(true)
        }
    }

    let conf = Arc::new(client::Config::default());
    let mut session = client::connect(conf, (cfg.host.as_str(), cfg.port), AcceptAll)
        .await
        .map_err(|e| format!("connect: {e}"))?;

    let auth = session
        .authenticate_password(&cfg.username, &cfg.password)
        .await
        .map_err(|e| format!("auth: {e}"))?;
    if !auth.success() {
        return Err("password authentication failed".into());
    }

    let mut channel = session
        .channel_open_session()
        .await
        .map_err(|e| format!("channel: {e}"))?;

    channel
        .request_pty(
            false,
            "xterm-256color",
            cfg.cols as u32,
            cfg.rows as u32,
            0,
            0,
            &[],
        )
        .await
        .map_err(|e| format!("pty: {e}"))?;
    channel
        .request_shell(true)
        .await
        .map_err(|e| format!("shell: {e}"))?;

    let _ = events.send(SshEvent::Connected);
    let _ = events.send(SshEvent::Status(format!(
        "ssh {}:{} ready",
        cfg.host, cfg.port
    )));
    let _ = events.send(SshEvent::Output(
        b"\x1b[2J\x1b[H\x1b[32m[ssh - remote echo]\x1b[0m\r\n".to_vec(),
    ));

    loop {
        tokio::select! {
            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(SshCmd::Data(data)) => {
                        if channel.data(&data[..]).await.is_err() {
                            break;
                        }
                    }
                    Some(SshCmd::Resize { cols, rows }) => {
                        let _ = channel.window_change(cols, rows, 0, 0).await;
                    }
                    Some(SshCmd::Close) | None => {
                        let _ = channel.eof().await;
                        let _ = session
                            .disconnect(Disconnect::ByApplication, "", "en")
                            .await;
                        break;
                    }
                }
            }
            msg = channel.wait() => {
                match msg {
                    Some(ChannelMsg::Data { ref data }) => {
                        let _ = events.send(SshEvent::Output(data.to_vec()));
                    }
                    Some(ChannelMsg::ExtendedData { ref data, .. }) => {
                        let _ = events.send(SshEvent::Output(data.to_vec()));
                    }
                    Some(ChannelMsg::Eof) | None => break,
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
