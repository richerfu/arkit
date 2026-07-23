//! Cooked local host for the demo — not part of arkit_terminal.

pub struct LocalShell {
    pub cols: u16,
    pub rows: u16,
    line: String,
    pending: Vec<u8>,
}

impl LocalShell {
    pub fn new(cols: u16, rows: u16) -> Self {
        Self {
            cols,
            rows,
            line: String::new(),
            pending: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.line.clear();
        self.pending.clear();
    }

    pub fn banner(&self) -> Vec<u8> {
        let mut o = Vec::new();
        o.extend_from_slice(b"\x1b[2J\x1b[H");
        o.extend_from_slice(b"\x1b[1;36markit terminal\x1b[0m\r\n");
        o.extend_from_slice(b"\x1b[2mhelp  echo  clear  uname  info\x1b[0m\r\n");
        // Enough lines that scrollback is exercisable on a 16-row viewport.
        for i in 1..=40 {
            let line = format!("history {i:02}\r\n");
            o.extend_from_slice(line.as_bytes());
        }
        o.extend_from_slice(b"\x1b[33mready\x1b[0m\r\n");
        o.extend(self.prompt());
        o
    }

    pub fn prompt(&self) -> Vec<u8> {
        b"\x1b[32m$\x1b[0m ".to_vec()
    }

    pub fn input(&mut self, data: &[u8]) -> Vec<u8> {
        self.pending.extend_from_slice(data);
        let mut out = Vec::new();
        loop {
            if self.pending.is_empty() {
                break;
            }
            let b = self.pending[0];
            if b == 0x1b {
                if let Some(n) = skip_escape(&self.pending) {
                    self.pending.drain(..n);
                    continue;
                }
                break;
            }
            match b {
                b'\r' | b'\n' => {
                    self.pending.remove(0);
                    if b == b'\r' && self.pending.first() == Some(&b'\n') {
                        self.pending.remove(0);
                    }
                    out.extend_from_slice(b"\r\n");
                    let cmd = self.line.trim().to_string();
                    self.line.clear();
                    out.extend(self.exec(&cmd));
                    out.extend(self.prompt());
                }
                0x7f | 0x08 => {
                    self.pending.remove(0);
                    if let Some(ch) = self.line.pop() {
                        let cells = if east_asian_wide(ch) { 2 } else { 1 };
                        for _ in 0..cells {
                            out.extend_from_slice(b"\x08\x1b[P");
                        }
                    }
                }
                0x03 => {
                    self.pending.remove(0);
                    self.line.clear();
                    out.extend_from_slice(b"^C\r\n");
                    out.extend(self.prompt());
                }
                0x0c => {
                    self.pending.remove(0);
                    out.extend_from_slice(b"\x1b[2J\x1b[H");
                    out.extend(self.prompt());
                    out.extend_from_slice(self.line.as_bytes());
                }
                c if (0x20..0x7f).contains(&c) => {
                    self.pending.remove(0);
                    self.line.push(c as char);
                    out.push(c);
                }
                c if c >= 0x80 => match std::str::from_utf8(&self.pending) {
                    Ok(s) => {
                        if let Some(ch) = s.chars().next() {
                            let n = ch.len_utf8();
                            self.pending.drain(..n);
                            self.line.push(ch);
                            let mut buf = [0u8; 4];
                            out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                        } else {
                            break;
                        }
                    }
                    Err(e) => {
                        let valid = e.valid_up_to();
                        if valid > 0 {
                            if let Ok(s) = std::str::from_utf8(&self.pending[..valid]) {
                                if let Some(ch) = s.chars().next() {
                                    let n = ch.len_utf8();
                                    self.pending.drain(..n);
                                    self.line.push(ch);
                                    let mut buf = [0u8; 4];
                                    out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                                    continue;
                                }
                            }
                        }
                        if e.error_len().is_some() {
                            self.pending.remove(0);
                        } else {
                            break;
                        }
                    }
                },
                _ => {
                    self.pending.remove(0);
                }
            }
        }
        out
    }

    fn exec(&self, cmd: &str) -> Vec<u8> {
        if cmd.is_empty() {
            return Vec::new();
        }
        let mut parts = cmd.split_whitespace();
        let head = parts.next().unwrap_or("");
        let rest: Vec<&str> = parts.collect();
        match head {
            "help" | "?" => b"help  echo  clear  uname  info\r\n".to_vec(),
            "echo" => format!("{}\r\n", rest.join(" ")).into_bytes(),
            "clear" => b"\x1b[2J\x1b[H".to_vec(),
            "uname" => b"arkit-local\r\n".to_vec(),
            "info" => format!("cols={} rows={}\r\n", self.cols, self.rows).into_bytes(),
            other => format!("not found: {other}\r\n").into_bytes(),
        }
    }
}

fn east_asian_wide(c: char) -> bool {
    let u = c as u32;
    matches!(
        u,
        0x1100..=0x115F
            | 0x2E80..=0xA4CF
            | 0xAC00..=0xD7A3
            | 0xF900..=0xFAFF
            | 0xFE10..=0xFE6F
            | 0xFF01..=0xFF60
            | 0xFFE0..=0xFFE6
            | 0x1F300..=0x1FAFF
            | 0x20000..=0x3FFFD
    )
}

fn skip_escape(buf: &[u8]) -> Option<usize> {
    if buf.is_empty() || buf[0] != 0x1b {
        return None;
    }
    if buf.len() == 1 {
        return None;
    }
    if buf[1] == b'O' {
        return if buf.len() >= 3 { Some(3) } else { None };
    }
    if buf[1] == b'[' {
        let mut i = 2;
        while i < buf.len() {
            if (0x40..=0x7e).contains(&buf[i]) {
                return Some(i + 1);
            }
            i += 1;
        }
        return None;
    }
    Some(2)
}
