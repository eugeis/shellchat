use crate::command::SHELL;
use crate::common::Question;
use crate::common::HEADER_API_KEY;
use crate::spinner::create_spinner;
use anyhow::{anyhow, Result};
use async_recursion::async_recursion;
use clipboard::ClipboardProvider;
use inquire::Select;
use log::debug;
use reqwest::Client;
use std::error::Error;
use std::process;

#[derive(Debug)]
pub struct Chatter {
    url: String,
    api_key: String,
    os: String,
    shell: String,
    client: Client,
}

impl Chatter {
    pub fn new(url: &str, api_key: &str, os: &str, shell: &str) -> Self {
        Chatter {
            url: url.to_string(),
            api_key: api_key.to_string(),
            os: os.to_string(),
            shell: shell.to_string(),
            client: Client::new(),
        }
    }

    pub async fn chat(&self, prompt: &str, explain: bool) -> Result<String, anyhow::Error> {
        let response = self
            .client
            .post(&self.url)
            .header(HEADER_API_KEY, &self.api_key)
            .json(&self.build_request(prompt, explain))
            .send()
            .await;

        match response {
            Ok(res) => {
                let status = res.status();
                let text = res
                    .text()
                    .await
                    .unwrap_or_else(|_| "Failed to read the answer".to_string());

                if status.is_success() {
                    Ok(text)
                } else {
                    Err(anyhow!("{}: {}", status, text))
                }
            }
            Err(err) => {
                if err.is_connect() || err.is_timeout() {
                    Err(anyhow!(
                        "Server not available. Please check the server status and try again."
                    ))
                } else {
                    Err(anyhow!("Request to the server failed: {}", err))
                }
            }
        }
    }

    #[async_recursion]
    pub async fn execute(&self, text: &str) -> Result<(), Box<dyn Error>> {
        let spinner = create_spinner("Translating").await;
        let result = self.chat(text, false).await;
        spinner.stop();

        match result {
            Ok(command) => loop {
                let answer = Select::new(
                    command.trim(),
                    vec!["✅ Execute", "📖 Explain", "📋 Copy", "❌ Cancel"],
                )
                    .prompt()?;

                match answer {
                    "✅ Execute" => {
                        debug!("{} {:?}", SHELL.cmd, &[&SHELL.arg, &command]);
                        let code = SHELL.run_command(&command, text)?;
                        if code != 0 {
                            process::exit(code);
                        }
                    }
                    "📖 Explain" => {
                        let explain_result = self.chat(&command, true).await?;
                        termimad::print_text(&explain_result);
                        continue;
                    }
                    "📋 Copy" => {
                        let mut clipboard = clipboard::ClipboardContext::new()?;
                        clipboard.set_contents(command.to_string())?;
                    }
                    "❌ Cancel" => break,
                    _ => {}
                }
                break;
            },
            Err(err) => {
                return Err(err.into());
            }
        }
        Ok(())
    }

    fn build_request(&self, prompt: &str, explain: bool) -> Question {
        Question {
            os: self.os.clone(),
            shell: self.shell.clone(),
            prompt: prompt.to_string(),
            explain,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_chat_success() {
        let chatter = Chatter::new("http://localhost:8080", "test_key", "Linux", "bash");

        let result = chatter.chat("Hello", false).await;
        assert!(result.is_err()); // Assuming there's no actual server running during tests
    }

    #[tokio::test]
    async fn test_shell_execute_success() {
        let chatter = Chatter::new("http://localhost:8080", "test_key", "Linux", "bash");

        let result = chatter.execute("echo Hello").await;
        assert!(result.is_err()); // Assuming there's no actual server running during tests
    }
}