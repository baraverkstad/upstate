use colored::Colorize;
use core::fmt::Display;

pub enum Format {
    Text,
    Json { sep: bool, depth: usize },
}

impl Format {
    pub fn json() -> Format {
        Format::Json { sep: false, depth: 0 }
    }

    pub fn text_summary(&self, key: &str, value: &str, detail: &str) {
        if let Format::Text = *self {
            println!("{:<10}{value:<26} {}", key.white(), detail.white());
        }
    }

    pub fn text_proc_ok(&self, label: String, detail: String) {
        if let Format::Text = *self {
            println!("{} {label:<34} {}", "\u{25CF}".green(), detail.white());
        }
    }

    pub fn text_proc_warn(&self, label: String, detail: String) {
        if let Format::Text = *self {
            println!("{} {label:<34} {}", "\u{25A0}".yellow(), detail.white());
        }
    }

    pub fn text_proc_err(&self, label: String, detail: String) {
        if let Format::Text = *self {
            println!("{} {label:<34} {}", "\u{25A0}".red(), detail);
        }
    }

    pub fn text_proc_more(&self, label: &str, message: String) {
        if let Format::Text = *self {
            println!("  {} {}", label.yellow(), message);
        }
    }

    pub fn json_open(&mut self, name: &str, array: bool, _newline: bool) {
        if let Format::Json { sep, depth } = *self {
            if sep {
                print!(",");
            }
            if !name.is_empty() {
                print!("\"{name}\":");
            }
            print!("{}", if array { "[" } else { "{" });
            *self = Format::Json { sep: false, depth: depth + 1 };
        }
    }

    pub fn json_close(&mut self, array: bool) {
        print!("{}", if array { "]" } else { "}" });
        if let Format::Json { depth, .. } = *self {
            *self = Format::Json { sep: true, depth: depth - 1 };
            if depth == 1 {
                println!();
            }
        }
    }

    pub fn json_key_val<T: Display>(&mut self, key: &str, value: T) {
        if let Format::Json { sep, depth } = *self {
            if sep {
                print!(",");
            }
            print!("\"{key}\":{value}");
            *self = Format::Json { sep: true, depth };
        }
    }

    pub fn json_key_str<T: Display>(&mut self, key: &str, value: T) {
        self.json_key_val(key, format!("\"{}\"", value));
    }
}
