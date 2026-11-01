use colored::Colorize;
use core::fmt::Display;
use std::io::{self, Write};

enum Mode {
    Text,
    Json { sep: bool, depth: usize },
}

pub struct Format<'a> {
    mode: Mode,
    output: Box<dyn Write + 'a>,
    json_buffer: Vec<u8>,
}

impl<'a> Format<'a> {
    pub fn text() -> Self {
        Self { mode: Mode::Text, output: Box::new(io::stdout()), json_buffer: Vec::new() }
    }

    pub fn json() -> Self {
        Self {
            mode: Mode::Json { sep: false, depth: 0 },
            output: Box::new(io::stdout()),
            json_buffer: Vec::new(),
        }
    }

    #[cfg(test)]
    pub fn with_output(mut self, output: impl Write + 'a) -> Self {
        self.output = Box::new(output);
        self
    }

    pub fn finish(&mut self) {
        if let Mode::Json { .. } = self.mode {
            self.output.write_all(&self.json_buffer).expect("failed writing report");
        }
    }

    pub fn text_summary(&mut self, key: &str, value: &str, detail: &str) {
        if let Mode::Text = self.mode {
            writeln!(self.output, "{:<10}{value:<26} {}", key.white(), detail.white()).expect("failed writing report");
        }
    }

    pub fn text_proc_ok(&mut self, label: String, detail: String) {
        if let Mode::Text = self.mode {
            writeln!(self.output, "{} {label:<34} {}", "\u{25CF}".green(), detail.white())
                .expect("failed writing report");
        }
    }

    pub fn text_proc_warn(&mut self, label: String, detail: String) {
        if let Mode::Text = self.mode {
            writeln!(self.output, "{} {label:<34} {}", "\u{25A0}".yellow(), detail.white())
                .expect("failed writing report");
        }
    }

    pub fn text_proc_err(&mut self, label: String, detail: String) {
        if let Mode::Text = self.mode {
            writeln!(self.output, "{} {label:<34} {}", "\u{25A0}".red(), detail).expect("failed writing report");
        }
    }

    pub fn text_proc_more(&mut self, label: &str, message: String) {
        if let Mode::Text = self.mode {
            writeln!(self.output, "  {} {}", label.yellow(), message).expect("failed writing report");
        }
    }

    pub fn json_open(&mut self, name: &str, array: bool, _newline: bool) {
        if let Mode::Json { sep, depth } = self.mode {
            if sep {
                write!(self.json_buffer, ",").expect("failed writing report");
            }
            if !name.is_empty() {
                serde_json::to_writer(&mut self.json_buffer, name).expect("failed writing report");
                write!(self.json_buffer, ":").expect("failed writing report");
            }
            write!(self.json_buffer, "{}", if array { "[" } else { "{" }).expect("failed writing report");
            self.mode = Mode::Json { sep: false, depth: depth + 1 };
        }
    }

    pub fn json_close(&mut self, array: bool) {
        if let Mode::Json { depth, .. } = self.mode {
            write!(self.json_buffer, "{}", if array { "]" } else { "}" }).expect("failed writing report");
            self.mode = Mode::Json { sep: true, depth: depth - 1 };
            if depth == 1 {
                writeln!(self.json_buffer).expect("failed writing report");
            }
        }
    }

    pub fn json_key_val<T: Display>(&mut self, key: &str, value: T) {
        if let Mode::Json { sep, depth } = self.mode {
            if sep {
                write!(self.json_buffer, ",").expect("failed writing report");
            }
            serde_json::to_writer(&mut self.json_buffer, key).expect("failed writing report");
            write!(self.json_buffer, ":{value}").expect("failed writing report");
            self.mode = Mode::Json { sep: true, depth };
        }
    }

    pub fn json_key_str<T: Display>(&mut self, key: &str, value: T) {
        let value = serde_json::to_string(&value.to_string()).expect("failed encoding report string");
        self.json_key_val(key, value);
    }
}
