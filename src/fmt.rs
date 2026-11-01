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
}

impl<'a> Format<'a> {
    pub fn text() -> Self {
        Self { mode: Mode::Text, output: Box::new(io::stdout()) }
    }

    pub fn json() -> Self {
        Self { mode: Mode::Json { sep: false, depth: 0 }, output: Box::new(io::stdout()) }
    }

    #[cfg(test)]
    pub fn with_output(mut self, output: impl Write + 'a) -> Self {
        self.output = Box::new(output);
        self
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
                write!(self.output, ",").expect("failed writing report");
            }
            if !name.is_empty() {
                write!(self.output, "\"{name}\":").expect("failed writing report");
            }
            write!(self.output, "{}", if array { "[" } else { "{" }).expect("failed writing report");
            self.mode = Mode::Json { sep: false, depth: depth + 1 };
        }
    }

    pub fn json_close(&mut self, array: bool) {
        write!(self.output, "{}", if array { "]" } else { "}" }).expect("failed writing report");
        if let Mode::Json { depth, .. } = self.mode {
            self.mode = Mode::Json { sep: true, depth: depth - 1 };
            if depth == 1 {
                writeln!(self.output,).expect("failed writing report");
            }
        }
    }

    pub fn json_key_val<T: Display>(&mut self, key: &str, value: T) {
        if let Mode::Json { sep, depth } = self.mode {
            if sep {
                write!(self.output, ",").expect("failed writing report");
            }
            write!(self.output, "\"{key}\":{value}").expect("failed writing report");
            self.mode = Mode::Json { sep: true, depth };
        }
    }

    pub fn json_key_str<T: Display>(&mut self, key: &str, value: T) {
        self.json_key_val(key, format!("\"{}\"", value));
    }
}
