use crate::options::ColorMode;
use std::io::IsTerminal;

pub struct Ui {
    colors_enabled: bool,
}

impl Ui {
    pub fn new(color_mode: ColorMode) -> Self {
        let colors_enabled = match color_mode {
            ColorMode::Auto => std::io::stdout().is_terminal(),
            ColorMode::Always => true,
            ColorMode::Never => false,
        };

        Self { colors_enabled }
    }

    pub fn section(&self, title: &str) -> String {
        self.style(&format!("== {} ==", title), "1;36")
    }

    pub fn info(&self, message: &str) -> String {
        format!("{} {}", self.style("[info]", "1;34"), message)
    }

    pub fn success(&self, message: &str) -> String {
        format!("{} {}", self.style("[ok]", "1;32"), message)
    }

    pub fn warning(&self, message: &str) -> String {
        format!("{} {}", self.style("[warn]", "1;33"), message)
    }

    pub fn prompt(&self, message: &str) -> String {
        self.style(message, "1")
    }

    pub fn key_value(&self, key: &str, value: &str) -> String {
        format!("  {:<24} {}", key, value)
    }

    pub fn style(&self, text: &str, code: &str) -> String {
        if self.colors_enabled {
            format!("\u{1b}[{code}m{text}\u{1b}[0m")
        } else {
            text.to_owned()
        }
    }
}
