//! OS PTY transport owned by the example, independently of terminal rendering.
//! Wakeable polling, startup diagnostics and resize acknowledgements follow
//! the local ohostty implementation; output uses the app's Tokio channels.

use std::fs::File;
use std::io::{self, Read, Write};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

use tokio::sync::mpsc as async_mpsc;

/// Identifies the failed startup operation while retaining the original errno.
#[derive(Debug)]
pub struct PtyStartError {
    operation: &'static str,
    source: io::Error,
}

impl PtyStartError {
    fn new(operation: &'static str, source: io::Error) -> Self {
        Self { operation, source }
    }

    fn at(operation: &'static str) -> impl FnOnce(io::Error) -> Self {
        move |source| Self::new(operation, source)
    }

    pub fn operation(&self) -> &'static str {
        self.operation
    }

    pub fn io_error(&self) -> &io::Error {
        &self.source
    }
}

impl std::fmt::Display for PtyStartError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.operation, self.source)
    }
}

impl std::error::Error for PtyStartError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

#[derive(Debug)]
pub enum PtyEvent {
    Started(u32),
    Output(Vec<u8>),
    Resized(u16, u16),
    Closed(String),
}

enum PtyCommand {
    Input(Vec<u8>),
    Resize(u16, u16),
}

pub struct LocalPty {
    commands: mpsc::SyncSender<PtyCommand>,
    stopped: Arc<AtomicBool>,
    wake: UnixStream,
}

impl LocalPty {
    pub fn start(
        home: PathBuf,
        cols: u16,
        rows: u16,
    ) -> Result<(Self, async_mpsc::Receiver<PtyEvent>), PtyStartError> {
        #[cfg(target_env = "ohos")]
        let shell = PathBuf::from("/system/bin/sh");
        #[cfg(not(target_env = "ohos"))]
        let shell = PathBuf::from("/bin/sh");
        let process = PtyProcess::start(shell, home, cols, rows)?;
        let (wake, worker_wake) =
            UnixStream::pair().map_err(PtyStartError::at("create PTY wake channel"))?;
        wake.set_nonblocking(true)
            .map_err(PtyStartError::at("configure PTY wake sender"))?;
        worker_wake
            .set_nonblocking(true)
            .map_err(PtyStartError::at("configure PTY wake receiver"))?;
        let (commands, receiver) = mpsc::sync_channel(64);
        // Bound output memory and apply backpressure without losing VT bytes.
        let (events, output) = async_mpsc::channel(64);
        let stopped = Arc::new(AtomicBool::new(false));
        let worker_stopped = stopped.clone();
        thread::Builder::new()
            .name("arkit-local-pty".into())
            .spawn(move || process.run(receiver, events, worker_stopped, worker_wake))
            .map_err(PtyStartError::at("start PTY worker"))?;
        Ok((
            Self {
                commands,
                stopped,
                wake,
            },
            output,
        ))
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
            })?;
        self.wake_worker();
        Ok(())
    }

    fn wake_worker(&self) {
        // A full socket already contains a wakeup. Commands remain in their
        // bounded queue; the socket only interrupts an otherwise idle poll.
        loop {
            match (&self.wake).write(&[1]) {
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                _ => break,
            }
        }
    }
}

impl Drop for LocalPty {
    fn drop(&mut self) {
        // Cancellation cannot sit behind a large paste in the input queue.
        self.stopped.store(true, Ordering::Release);
        self.wake_worker();
    }
}

struct PtyProcess {
    master: Option<File>,
    child: Child,
}

impl PtyProcess {
    fn start(shell: PathBuf, home: PathBuf, cols: u16, rows: u16) -> Result<Self, PtyStartError> {
        if cols == 0 || rows == 0 {
            return Err(PtyStartError::new(
                "validate PTY grid",
                io::Error::new(io::ErrorKind::InvalidInput, "empty PTY grid"),
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
            return Err(PtyStartError::new(
                "allocate PTY",
                io::Error::last_os_error(),
            ));
        }
        // SAFETY: openpty returned two distinct owned descriptors.
        let master = unsafe { File::from_raw_fd(master) };
        // SAFETY: the slave descriptor has not been transferred elsewhere.
        let slave = unsafe { File::from_raw_fd(slave) };
        for fd in [master.as_raw_fd(), slave.as_raw_fd()] {
            // SAFETY: fd is live; setting CLOEXEC prevents leakage into exec.
            if unsafe { libc::fcntl(fd, libc::F_SETFD, libc::FD_CLOEXEC) } < 0 {
                return Err(PtyStartError::new(
                    "configure PTY descriptor",
                    io::Error::last_os_error(),
                ));
            }
        }
        let mut command = Command::new(shell);
        command
            .arg("-i")
            .env("TERM", "xterm-256color")
            .stdin(Stdio::from(
                slave
                    .try_clone()
                    .map_err(PtyStartError::at("duplicate PTY stdin"))?,
            ))
            .stdout(Stdio::from(
                slave
                    .try_clone()
                    .map_err(PtyStartError::at("duplicate PTY stdout"))?,
            ))
            .stderr(Stdio::from(slave));
        {
            // AppSpawn starts native apps at the sandbox root, whose contents
            // are not readable by an ordinary app. Start in app-owned storage
            // so commands such as `ls`, redirection and file creation work.
            std::fs::create_dir_all(&home).map_err(PtyStartError::at("create terminal home"))?;
            command
                .current_dir(&home)
                .env("HOME", &home)
                .env("COLORTERM", "truecolor")
                .env("TERM_PROGRAM", "arkit");
        }
        #[cfg(target_env = "ohos")]
        command.env("PATH", "/system/bin:/vendor/bin");
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
        let child = command.spawn().map_err(PtyStartError::at("spawn shell"))?;
        let process = Self {
            master: Some(master),
            child,
        };
        // SAFETY: the owned master fd is live; preserve existing status flags.
        let flags = unsafe { libc::fcntl(process.master_fd(), libc::F_GETFL) };
        if flags < 0 {
            return Err(PtyStartError::new(
                "read PTY status flags",
                io::Error::last_os_error(),
            ));
        }
        // SAFETY: the same descriptor remains live throughout this operation.
        if unsafe { libc::fcntl(process.master_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0
        {
            return Err(PtyStartError::new(
                "configure nonblocking PTY",
                io::Error::last_os_error(),
            ));
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
        events: async_mpsc::Sender<PtyEvent>,
        stopped: Arc<AtomicBool>,
        mut wake: UnixStream,
    ) {
        let result = self.pump(&commands, &events, &stopped, &mut wake);
        let message = match result {
            Ok(message) => message,
            Err(error) => format!("PTY error: {error}"),
        };
        let mut event = PtyEvent::Closed(message);
        while !stopped.load(Ordering::Acquire) {
            match events.try_send(event) {
                Ok(()) | Err(async_mpsc::error::TrySendError::Closed(_)) => break,
                Err(async_mpsc::error::TrySendError::Full(returned)) => event = returned,
            }
            thread::sleep(Duration::from_millis(8));
        }
    }

    fn pump(
        &mut self,
        commands: &mpsc::Receiver<PtyCommand>,
        events: &async_mpsc::Sender<PtyEvent>,
        stopped: &AtomicBool,
        wake: &mut UnixStream,
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
                    Err(async_mpsc::error::TrySendError::Full(event)) => {
                        pending_event = Some(event)
                    }
                    Err(async_mpsc::error::TrySendError::Closed(_)) => {
                        return Ok("PTY detached".into())
                    }
                }
            }
            if pending_event.is_some() {
                thread::sleep(Duration::from_millis(8));
                continue;
            }
            if pending_write.is_none() {
                match commands.try_recv() {
                    Ok(PtyCommand::Input(data)) => pending_write = Some((data, 0)),
                    Ok(PtyCommand::Resize(cols, rows)) => {
                        self.resize(cols, rows)?;
                        pending_event = Some(PtyEvent::Resized(cols, rows));
                        continue;
                    }
                    Err(mpsc::TryRecvError::Disconnected) => return Ok("PTY detached".into()),
                    Err(mpsc::TryRecvError::Empty) => {}
                }
            }
            let mut polls = [
                libc::pollfd {
                    fd: self.master_fd(),
                    events: libc::POLLIN
                        | if pending_write.is_some() {
                            libc::POLLOUT
                        } else {
                            0
                        },
                    revents: 0,
                },
                libc::pollfd {
                    fd: wake.as_raw_fd(),
                    events: libc::POLLIN,
                    revents: 0,
                },
            ];
            // SAFETY: both entries refer to live owned descriptors. Input and
            // cancellation explicitly wake this poll, so idle tabs need no timer.
            if unsafe { libc::poll(polls.as_mut_ptr(), 2, -1) } < 0 {
                let error = io::Error::last_os_error();
                if error.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(error);
            }
            if polls[1].revents != 0 {
                let mut signals = [0u8; 128];
                while wake.read(&mut signals).is_ok_and(|length| length > 0) {}
            }
            let poll = &polls[0];
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

    struct TestHome(PathBuf);

    impl TestHome {
        fn new() -> Self {
            let unique = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            Self(std::env::temp_dir().join(format!("arkit-pty-{}-{unique}", std::process::id())))
        }
    }

    impl Drop for TestHome {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn receive(rx: &mut async_mpsc::Receiver<PtyEvent>, timeout: Duration) -> PtyEvent {
        let deadline = Instant::now() + timeout;
        loop {
            match rx.try_recv() {
                Ok(event) => return event,
                Err(async_mpsc::error::TryRecvError::Disconnected) => panic!("PTY channel closed"),
                Err(async_mpsc::error::TryRecvError::Empty) => {}
            }
            assert!(Instant::now() < deadline, "PTY output timed out");
            thread::sleep(Duration::from_millis(1));
        }
    }

    fn until(rx: &mut async_mpsc::Receiver<PtyEvent>, output: &mut Vec<u8>, needle: &[u8]) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !output.windows(needle.len()).any(|part| part == needle) {
            let event = receive(rx, deadline.saturating_duration_since(Instant::now()));
            if let PtyEvent::Output(bytes) = event {
                output.extend(bytes);
            } else if let PtyEvent::Closed(reason) = event {
                panic!("PTY closed early: {reason}");
            }
        }
    }

    fn child_pid(rx: &mut async_mpsc::Receiver<PtyEvent>) -> u32 {
        match receive(rx, Duration::from_secs(5)) {
            PtyEvent::Started(pid) => pid,
            event => panic!("expected startup event, got {event:?}"),
        }
    }

    fn assert_reaped(pid: u32) {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            // SAFETY: signal zero checks existence without changing a process.
            if unsafe { libc::kill(pid as libc::pid_t, 0) } < 0 {
                assert_eq!(io::Error::last_os_error().raw_os_error(), Some(libc::ESRCH));
                break;
            }
            assert!(Instant::now() < deadline, "PTY child {pid} leaked");
            thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn startup_error_identifies_operation_and_preserves_errno() {
        let error = PtyProcess::start(
            PathBuf::from("/arkit-no-such-shell"),
            std::env::temp_dir(),
            40,
            24,
        )
        .err()
        .expect("missing shell must fail");
        assert_eq!(error.operation(), "spawn shell");
        assert_eq!(error.io_error().raw_os_error(), Some(libc::ENOENT));
        assert!(error.to_string().contains("spawn shell"));
        assert!(std::error::Error::source(&error).is_some());

        let error = LocalPty::start(std::env::temp_dir(), 0, 24)
            .err()
            .expect("empty grid must fail before allocation");
        assert_eq!(error.operation(), "validate PTY grid");
        assert_eq!(error.io_error().kind(), io::ErrorKind::InvalidInput);

        let error = PtyProcess::start(
            PathBuf::from("/bin/sh"),
            std::env::current_exe().unwrap().join("terminal"),
            40,
            24,
        )
        .err()
        .expect("home beneath a file must fail");
        assert_eq!(error.operation(), "create terminal home");
        assert_eq!(error.io_error().raw_os_error(), Some(libc::ENOTDIR));
    }

    #[test]
    fn shell_uses_configured_home_and_terminal_environment() {
        let home = TestHome::new();
        let (pty, mut rx) = LocalPty::start(home.0.clone(), 40, 24).unwrap();
        let mut output = Vec::new();
        pty.write(
            b"test \"$HOME\" -ef . && : > writable && printf 'HOME_%s\\n' READY; printf 'ENV|%s|%s|%s\\n' \"$TERM\" \"$COLORTERM\" \"$TERM_PROGRAM\"\r",
        )
        .unwrap();
        until(&mut rx, &mut output, b"HOME_READY");
        until(&mut rx, &mut output, b"ENV|xterm-256color|truecolor|arkit");
        assert!(home.0.join("writable").is_file());
    }

    #[test]
    fn queued_resizes_wake_idle_worker_and_acknowledge_in_order() {
        let (pty, mut rx) = LocalPty::start(std::env::temp_dir(), 40, 24).unwrap();
        pty.write(b"printf 'IDLE_%s\\n' READY\r").unwrap();
        until(&mut rx, &mut Vec::new(), b"IDLE_READY");
        thread::sleep(Duration::from_millis(50));
        for cols in 80..96 {
            pty.resize(cols, 30).unwrap();
        }
        for expected_cols in 80..96 {
            loop {
                match receive(&mut rx, Duration::from_secs(5)) {
                    PtyEvent::Resized(cols, rows) => {
                        assert_eq!((cols, rows), (expected_cols, 30));
                        break;
                    }
                    PtyEvent::Closed(reason) => panic!("PTY closed early: {reason}"),
                    _ => {}
                }
            }
        }
        pty.write(b"stty size\r").unwrap();
        until(&mut rx, &mut Vec::new(), b"30 95");
    }

    #[test]
    fn async_output_wakes_waiting_receiver() {
        let (pty, mut rx) = LocalPty::start(std::env::temp_dir(), 40, 24).unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap();
        runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(5), async {
                pty.write(b"sleep 1; printf 'ASYNC_%s\\n' READY\r").unwrap();
                let mut output = Vec::new();
                while let Some(event) = rx.recv().await {
                    match event {
                        PtyEvent::Output(bytes) => output.extend(bytes),
                        PtyEvent::Closed(reason) => panic!("PTY closed early: {reason}"),
                        _ => {}
                    }
                    if output
                        .windows(b"ASYNC_READY".len())
                        .any(|part| part == b"ASYNC_READY")
                    {
                        return;
                    }
                }
                panic!("PTY channel closed");
            })
            .await
            .expect("async PTY output timed out");
        });
    }

    #[test]
    fn dropping_idle_host_wakes_worker_and_reaps_child() {
        let (pty, mut rx) = LocalPty::start(std::env::temp_dir(), 40, 24).unwrap();
        let pid = child_pid(&mut rx);
        pty.write(b"printf 'IDLE_%s\\n' READY\r").unwrap();
        until(&mut rx, &mut Vec::new(), b"IDLE_READY");
        thread::sleep(Duration::from_millis(50));
        drop(pty);
        assert_reaped(pid);
    }

    #[test]
    fn real_tty_resize_interrupt_and_exit() {
        let (pty, mut rx) = LocalPty::start(std::env::temp_dir(), 40, 24).unwrap();
        assert_eq!(
            pty.resize(0, 24).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
        let mut output = Vec::new();
        pty.write(b"test -t 0 && printf 'TTY_%s\\n' READY; stty size\r")
            .unwrap();
        until(&mut rx, &mut output, b"TTY_READY");
        until(&mut rx, &mut output, b"24 40");
        output.clear();
        pty.resize(113, 37).unwrap();
        pty.write(b"stty size\r").unwrap();
        until(&mut rx, &mut output, b"37 113");
        pty.write(b"sleep 10\r").unwrap();
        thread::sleep(Duration::from_millis(150));
        output.clear();
        pty.write(b"\x03").unwrap();
        pty.write(b"printf 'AFTER_%s\\n' INTERRUPT\r").unwrap();
        until(&mut rx, &mut output, b"AFTER_INTERRUPT");
        pty.write(b"exit 7\r").unwrap();
        loop {
            if let PtyEvent::Closed(reason) = receive(&mut rx, Duration::from_secs(5)) {
                assert!(reason.contains('7'), "{reason}");
                break;
            }
        }
    }

    #[test]
    fn dropping_host_reaps_child_even_with_output_backpressure() {
        let (pty, mut rx) = LocalPty::start(std::env::temp_dir(), 40, 24).unwrap();
        let pid = child_pid(&mut rx);
        pty.write(b"while :; do printf 'output output output\\n'; done\r")
            .unwrap();
        thread::sleep(Duration::from_millis(200));
        drop(pty);
        assert_reaped(pid);
    }
}
