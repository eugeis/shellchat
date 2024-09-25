use anyhow::Result;
use std::io::IsTerminal;
use std::{env, process::Command};

lazy_static::lazy_static! {
    pub static ref OS: String = detect_os();
    pub static ref SHELL: Shell = detect_shell();
    pub static ref IS_STDOUT_TERMINAL: bool = std::io::stdout().is_terminal();
}

pub fn detect_os() -> String {
    let os = env::consts::OS;
    if os == "linux" {
        if let Ok(contents) = std::fs::read_to_string("/etc/os-release") {
            for line in contents.lines() {
                if let Some(id) = line.strip_prefix("ID=") {
                    return format!("{os} ({id})");
                }
            }
        }
    }
    os.to_string()
}

pub struct Shell {
    pub name: String,
    pub cmd: String,
    pub arg: String,
    pub history_cmd: Option<String>,
}

impl Shell {
    pub fn new(name: &str, cmd: &str, arg: &str, history_cmd: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            cmd: cmd.to_string(),
            arg: arg.to_string(),
            history_cmd: history_cmd.map(|cmd| cmd.to_string()),
        }
    }

    pub fn run_command(&self, eval_str: &str) -> Result<i32> {
        let status = Command::new(&self.cmd)
            .arg(&self.arg)
            .arg(eval_str)
            .status()?;

        if status.success() {
            if let Some(history_cmd) = &self.history_cmd {
                let _ = Command::new(&self.cmd)
                    .arg(&self.arg)
                    .arg(format!("{} \"{}\"", history_cmd, eval_str))
                    .status();
            }
        }

        Ok(status.code().unwrap_or_default())
    }
}

pub fn detect_shell() -> Shell {
    let os = env::consts::OS;
    if os == "windows" {
        if let Some(ret) = env::var("PSModulePath").ok().and_then(|v| {
            let v = v.to_lowercase();
            if v.split(';').count() >= 3 {
                if v.contains("powershell\\7\\") {
                    Some(Shell::new("pwsh", "pwsh.exe", "-c", None))
                } else {
                    Some(Shell::new("powershell", "powershell.exe", "-Command", None))
                }
            } else {
                None
            }
        }) {
            ret
        } else {
            Shell::new("cmd", "cmd.exe", "/C", None)
        }
    } else {
        let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let shell = match shell.rsplit_once('/') {
            Some((_, v)) => v,
            None => &shell,
        };
        match shell {
            "bash" => Shell::new(shell, shell, "-c", Some("history -s")),
            "zsh" => Shell::new(shell, shell, "-c", Some("print -s")),
            "fish" | "pwsh" => Shell::new(shell, shell, "-c", None),
            _ => Shell::new("sh", "sh", "-c", None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_os() {
        let os = detect_os();
        assert!(!os.is_empty());
    }

    #[test]
    fn test_detect_shell() {
        let shell = detect_shell();
        assert!(!shell.name.is_empty());
        assert!(!shell.cmd.is_empty());
        assert!(!shell.arg.is_empty());
    }

    #[test]
    fn test_run_command() {
        let result = SHELL.run_command("echo Hello, world!");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }
}