//! A real OS PTY host owned by the example, independently of terminal rendering.
//! Like HiShell's local path, this owns a PTY pair, an interactive child, a
//! nonblocking master, and foreground-process resize notifications.

use std::fs::File;
use std::io::{self, Read, Write};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub enum PtyEvent {
    Started(u32),
    Output(Vec<u8>),
    Closed(String),
}

enum PtyCommand {
    Input(Vec<u8>),
    Resize(u16, u16),
}

pub struct LocalPty {
    commands: mpsc::SyncSender<PtyCommand>,
    stopped: Arc<AtomicBool>,
}

impl LocalPty {
    pub fn start(cols: u16, rows: u16) -> io::Result<(Self, mpsc::Receiver<PtyEvent>)> {
        #[cfg(target_env = "ohos")]
        let shell = PathBuf::from("/system/bin/sh");
        #[cfg(not(target_env = "ohos"))]
        let shell = PathBuf::from("/bin/sh");
        let process = PtyProcess::start(shell, cols, rows)?;
        let (commands, receiver) = mpsc::sync_channel(64);
        // Bound output memory and apply backpressure without losing VT bytes.
        let (events, output) = mpsc::sync_channel(64);
        let stopped = Arc::new(AtomicBool::new(false));
        let worker_stopped = stopped.clone();
        thread::Builder::new()
            .name("arkit-local-pty".into())
            .spawn(move || process.run(receiver, events, worker_stopped))?;
        Ok((Self { commands, stopped }, output))
    }

    pub fn write(&self, bytes: &[u8]) -> io::Result<()> {
        self.send(PtyCommand::Input(bytes.to_vec()))
    }

    pub fn resize(&self, cols: u16, rows: u16) -> io::Result<()> {
        if cols == 0 || rows == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "empty PTY grid",
            ));
        }
        self.send(PtyCommand::Resize(cols, rows))
    }

    fn send(&self, command: PtyCommand) -> io::Result<()> {
        self.commands
            .try_send(command)
            .map_err(|error| match error {
                mpsc::TrySendError::Full(_) => {
                    io::Error::new(io::ErrorKind::WouldBlock, "PTY input queue full")
                }
                mpsc::TrySendError::Disconnected(_) => {
                    io::Error::new(io::ErrorKind::BrokenPipe, "PTY exited")
                }
            })
    }
}

impl Drop for LocalPty {
    fn drop(&mut self) {
        // Cancellation cannot sit behind a large paste in the input queue.
        self.stopped.store(true, Ordering::Release);
    }
}

struct PtyProcess {
    master: Option<File>,
    child: Child,
}

impl PtyProcess {
    fn start(shell: PathBuf, cols: u16, rows: u16) -> io::Result<Self> {
        if cols == 0 || rows == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "empty PTY grid",
            ));
        }
        let mut size = libc::winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        let mut master = -1;
        let mut slave = -1;
        // SAFETY: valid output pointers and winsize; null requests default
        // terminal attributes and no returned pathname. Successful descriptors
        // are transferred immediately into File ownership below.
        let rc = unsafe {
            libc::openpty(
                &mut master,
                &mut slave,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::addr_of_mut!(size),
            )
        };
        if rc < 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: openpty returned two distinct owned descriptors.
        let master = unsafe { File::from_raw_fd(master) };
        // SAFETY: the slave descriptor has not been transferred elsewhere.
        let slave = unsafe { File::from_raw_fd(slave) };
        for fd in [master.as_raw_fd(), slave.as_raw_fd()] {
            // SAFETY: fd is live; setting CLOEXEC prevents leakage into exec.
            if unsafe { libc::fcntl(fd, libc::F_SETFD, libc::FD_CLOEXEC) } < 0 {
                return Err(io::Error::last_os_error());
            }
        }
        let mut command = Command::new(shell);
        command
            .arg("-i")
            .env("TERM", "xterm-256color")
            .stdin(Stdio::from(slave.try_clone()?))
            .stdout(Stdio::from(slave.try_clone()?))
            .stderr(Stdio::from(slave));
        #[cfg(target_env = "ohos")]
        {
            // AppSpawn starts native apps at the sandbox root, whose contents
            // are not readable by an ordinary app. Start in app-owned storage
            // so commands such as `ls`, redirection and file creation work.
            let home = PathBuf::from("/data/storage/el2/base/files/terminal");
            std::fs::create_dir_all(&home)?;
            command
                .current_dir(&home)
                .env("HOME", &home)
                .env("PATH", "/system/bin:/vendor/bin");
        }
        // SAFETY: this callback runs between fork and exec and only invokes
        // async-signal-safe system operations. stdin already points at the
        // slave; the child becomes a session leader with a controlling tty.
        unsafe {
            command.pre_exec(|| {
                if libc::setsid() < 0 || libc::ioctl(0, libc::TIOCSCTTY as _, 0) < 0 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let child = command.spawn()?;
        let process = Self {
            master: Some(master),
            child,
        };
        // SAFETY: the owned master fd is live; preserve existing status flags.
        let flags = unsafe { libc::fcntl(process.master_fd(), libc::F_GETFL) };
        if flags < 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: the same descriptor remains live throughout this operation.
        if unsafe { libc::fcntl(process.master_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(process)
    }

    fn master_fd(&self) -> libc::c_int {
        self.master.as_ref().expect("live PTY master").as_raw_fd()
    }

    fn resize(&self, cols: u16, rows: u16) -> io::Result<()> {
        let size = libc::winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        // SAFETY: the owned master fd and the winsize pointer are valid.
        if unsafe { libc::ioctl(self.master_fd(), libc::TIOCSWINSZ, &size) } < 0 {
            return Err(io::Error::last_os_error());
        }
        // HiShell also explicitly notifies the foreground process group.
        // SAFETY: tcgetpgrp reads this PTY; only a positive group id is signalled.
        let group = unsafe { libc::tcgetpgrp(self.master_fd()) };
        if group > 0 {
            // SAFETY: a negative positive-pgid designates that foreground group.
            unsafe { libc::kill(-group, libc::SIGWINCH) };
        }
        Ok(())
    }

    fn run(
        mut self,
        commands: mpsc::Receiver<PtyCommand>,
        events: mpsc::SyncSender<PtyEvent>,
        stopped: Arc<AtomicBool>,
    ) {
        let result = self.pump(&commands, &events, &stopped);
        let message = match result {
            Ok(message) => message,
            Err(error) => format!("PTY error: {error}"),
        };
        let mut event = PtyEvent::Closed(message);
        while !stopped.load(Ordering::Acquire) {
            match events.try_send(event) {
                Ok(()) | Err(mpsc::TrySendError::Disconnected(_)) => break,
                Err(mpsc::TrySendError::Full(returned)) => event = returned,
            }
            thread::sleep(Duration::from_millis(8));
        }
    }

    fn pump(
        &mut self,
        commands: &mpsc::Receiver<PtyCommand>,
        events: &mpsc::SyncSender<PtyEvent>,
        stopped: &AtomicBool,
    ) -> io::Result<String> {
        let mut pending_event = Some(PtyEvent::Started(self.child.id()));
        let mut pending_write: Option<(Vec<u8>, usize)> = None;
        let mut eof = false;
        let mut bytes = [0u8; 8192];
        while !stopped.load(Ordering::Acquire) {
            if eof {
                if let Some(status) = self.child.try_wait()? {
                    return Ok(status.to_string());
                }
                // A process can close its tty while continuing to run. Keep
                // cancellation responsive instead of blocking in waitpid.
                thread::sleep(Duration::from_millis(8));
                continue;
            }
            if let Some(event) = pending_event.take() {
                match events.try_send(event) {
                    Ok(()) => {}
                    Err(mpsc::TrySendError::Full(event)) => pending_event = Some(event),
                    Err(mpsc::TrySendError::Disconnected(_)) => return Ok("PTY detached".into()),
                }
            }
            if pending_write.is_none() {
                match commands.try_recv() {
                    Ok(PtyCommand::Input(data)) => pending_write = Some((data, 0)),
                    Ok(PtyCommand::Resize(cols, rows)) => self.resize(cols, rows)?,
                    Err(mpsc::TryRecvError::Disconnected) => return Ok("PTY detached".into()),
                    Err(mpsc::TryRecvError::Empty) => {}
                }
            }
            if pending_event.is_some() {
                thread::sleep(Duration::from_millis(8));
                continue;
            }
            let mut poll = libc::pollfd {
                fd: self.master_fd(),
                events: libc::POLLIN
                    | if pending_write.is_some() {
                        libc::POLLOUT
                    } else {
                        0
                    },
                revents: 0,
            };
            // SAFETY: poll points at one live, initialized pollfd.
            if unsafe { libc::poll(&mut poll, 1, 16) } < 0 {
                let error = io::Error::last_os_error();
                if error.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(error);
            }
            if poll.revents & (libc::POLLIN | libc::POLLHUP | libc::POLLERR) != 0 {
                match self
                    .master
                    .as_mut()
                    .expect("live PTY master")
                    .read(&mut bytes)
                {
                    Ok(0) => eof = true,
                    Ok(length) => pending_event = Some(PtyEvent::Output(bytes[..length].to_vec())),
                    Err(error) if error.raw_os_error() == Some(libc::EIO) => {
                        eof = true;
                    }
                    Err(error)
                        if matches!(
                            error.kind(),
                            io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                        ) => {}
                    Err(error) => return Err(error),
                }
            }
            if poll.revents & libc::POLLOUT != 0 {
                if let Some((data, offset)) = &mut pending_write {
                    match self
                        .master
                        .as_mut()
                        .expect("live PTY master")
                        .write(&data[*offset..])
                    {
                        Ok(length) => {
                            *offset += length;
                            if *offset == data.len() {
                                pending_write = None;
                            }
                        }
                        Err(error)
                            if matches!(
                                error.kind(),
                                io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                            ) => {}
                        Err(error) => return Err(error),
                    }
                }
            }
        }
        Ok("PTY closed".into())
    }
}

impl Drop for PtyProcess {
    fn drop(&mut self) {
        if !matches!(self.child.try_wait(), Ok(Some(_))) {
            // Closing the master releases a child blocked in tty output
            // before waitpid, including when the UI output queue is full.
            self.master.take();
            // SAFETY: the unreaped child still owns this pid/session. Hang up
            // its shell group before killing and reaping the direct child.
            unsafe { libc::kill(-(self.child.id() as libc::pid_t), libc::SIGHUP) };
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn until(rx: &mpsc::Receiver<PtyEvent>, output: &mut Vec<u8>, needle: &[u8]) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !output.windows(needle.len()).any(|part| part == needle) {
            let event = rx
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .expect("PTY output timed out");
            if let PtyEvent::Output(bytes) = event {
                output.extend(bytes);
            } else if let PtyEvent::Closed(reason) = event {
                panic!("PTY closed early: {reason}");
            }
        }
    }

    #[test]
    fn real_tty_resize_interrupt_and_exit() {
        let (pty, rx) = LocalPty::start(40, 24).unwrap();
        let mut output = Vec::new();
        pty.write(b"test -t 0 && printf 'TTY_%s\\n' READY; stty size\r")
            .unwrap();
        until(&rx, &mut output, b"TTY_READY");
        until(&rx, &mut output, b"24 40");
        output.clear();
        pty.resize(113, 37).unwrap();
        pty.write(b"stty size\r").unwrap();
        until(&rx, &mut output, b"37 113");
        pty.write(b"sleep 10\r").unwrap();
        thread::sleep(Duration::from_millis(150));
        output.clear();
        pty.write(b"\x03").unwrap();
        pty.write(b"printf 'AFTER_%s\\n' INTERRUPT\r").unwrap();
        until(&rx, &mut output, b"AFTER_INTERRUPT");
        pty.write(b"exit 7\r").unwrap();
        loop {
            if let PtyEvent::Closed(reason) = rx.recv_timeout(Duration::from_secs(5)).unwrap() {
                assert!(reason.contains('7'), "{reason}");
                break;
            }
        }
    }

    #[test]
    fn dropping_host_reaps_child_even_with_output_backpressure() {
        let (pty, rx) = LocalPty::start(40, 24).unwrap();
        let pid = loop {
            if let PtyEvent::Started(pid) = rx.recv_timeout(Duration::from_secs(5)).unwrap() {
                break pid;
            }
        };
        pty.write(b"while :; do printf 'output output output\\n'; done\r")
            .unwrap();
        thread::sleep(Duration::from_millis(200));
        drop(pty);
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            // SAFETY: signal zero checks existence without changing a process.
            if unsafe { libc::kill(pid as libc::pid_t, 0) } < 0 {
                break;
            }
            assert!(Instant::now() < deadline, "PTY child {pid} leaked");
            thread::sleep(Duration::from_millis(10));
        }
    }
}
