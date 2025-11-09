use colored::Colorize;
use core::fmt::Display;

pub enum Format {
    Text,
    Json { indent: usize, nl: bool, sep: bool },
}

impl Format {
    pub fn json() -> Format {
        Format::Json { indent: 0, nl: true, sep: false }
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
            println!("  {}: {}", label.yellow(), message);
        }
    }

    pub fn json_open(&mut self, name: &str, array: bool, newline: bool) {
        if let Format::Json { indent, nl, sep } = *self {
            self.json_sep(sep, nl && indent > 0, !nl && indent > 0, indent);
            if !name.is_empty() {
                print!("\"{name}\": ");
            }
            print!("{}", if array { "[" } else { "{" });
        }
        if let Format::Json { ref mut indent, ref mut nl, ref mut sep } = *self {
            *indent += 2;
            *nl = newline;
            *sep = false;
        }
    }

    pub fn json_close(&mut self, array: bool) {
        if let Format::Json { indent, nl, .. } = *self {
            self.json_sep(false, nl, !nl, indent - 2);
            print!("{}", if array { "]" } else { "}" });
        }
        if let Format::Json { ref mut indent, ref mut nl, .. } = *self {
            *indent -= 2;
            *nl = true;
            if *indent == 0 {
                println!();
            }
        }
    }

    pub fn json_key_val<T: Display>(&mut self, key: &str, value: T) {
        if let Format::Json { indent, nl, sep, .. } = *self {
            self.json_sep(sep, nl, !nl, indent);
            print!("\"{key}\": {value}");
        }
        if let Format::Json { ref mut sep, .. } = *self {
            *sep = true;
        }
    }

    pub fn json_key_str<T: Display>(&mut self, key: &str, value: T) {
        self.json_key_val(key, format!("\"{}\"", value));
    }

    fn json_sep(&self, sep: bool, nl: bool, sp: bool, indent: usize) {
        print!("{}", if sep { "," } else { "" });
        if nl {
            print!("\n{:indent$}", "");
        } else if sp {
            print!(" ");
        }
    }
}
