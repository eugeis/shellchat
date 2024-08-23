use anyhow::Result;
use std::collections::HashMap;
use std::io::IsTerminal;
use std::{env, ffi::OsStr, process::Command};

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
}

impl Shell {
    pub fn new(name: &str, cmd: &str, arg: &str) -> Self {
        Self {
            name: name.to_string(),
            cmd: cmd.to_string(),
            arg: arg.to_string(),
        }
    }
}

pub fn detect_shell() -> Shell {
    let os = env::consts::OS;
    if os == "windows" {
        if let Some(ret) = env::var("PSModulePath").ok().and_then(|v| {
            let v = v.to_lowercase();
            if v.split(';').count() >= 3 {
                if v.contains("powershell\\7\\") {
                    Some(Shell::new("pwsh", "pwsh.exe", "-c"))
                } else {
                    Some(Shell::new("powershell", "powershell.exe", "-Command"))
                }
            } else {
                None
            }
        }) {
            ret
        } else {
            Shell::new("cmd", "cmd.exe", "/C")
        }
    } else {
        let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let shell = match shell.rsplit_once('/') {
            Some((_, v)) => v,
            None => &shell,
        };
        match shell {
            "bash" | "zsh" | "fish" | "pwsh" => Shell::new(shell, shell, "-c"),
            _ => Shell::new("sh", "sh", "-c"),
        }
    }
}

pub fn run_command<T: AsRef<OsStr>>(
    cmd: &str,
    args: &[T],
    envs: Option<HashMap<String, String>>,
) -> Result<i32> {
    let status = Command::new(cmd)
        .args(args.iter())
        .envs(envs.unwrap_or_default())
        .status()?;
    Ok(status.code().unwrap_or_default())
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
        let result = run_command("echo", &["Hello, world!"], None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }
}
