use crate::command;
use crate::command::run_command;
use crate::command::SHELL;
use crate::common::Question;
use crate::common::Answer;
use crate::common::HEADER_API_KEY;
use crate::spinner::create_spinner;
use inquire::Select;
use log::debug;
use reqwest::Client;
use std::process;
use anyhow::{anyhow, Result};


#[derive(Debug)]
pub struct Chatter {
    url: String,
    shell: String,
    os: String,
    client: Client,
    api_key: String,
}

impl Chatter {
    pub fn new(url: &str, api_key: &str) -> Self {
        Chatter {
            url: url.to_string(),
            shell: SHELL.name.clone(),
            os: String::from(command::OS.as_str()), // Using OS directly
            client: Client::new(),
            api_key: api_key.to_string(),
        }
    }

    async fn chat(&self, prompt: &str, explain: bool) -> Result<Answer> {
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
                let text = res.text().await.unwrap_or_else(|_| "Failed to read response text".to_string());

                if status.is_success() {
                    let json: Result<Answer, _> = serde_json::from_str(&text);
                    match json {
                        Ok(answer) => Ok(answer),
                        Err(err) => Err(anyhow!("Failed to parse server response: {}\nResponse text: {}", err, text)),
                    }
                } else {
                    Err(anyhow!("Server returned an error: {}\nResponse text: {}", status, text))
                }
            }
            Err(err) => {
                if err.is_connect() || err.is_timeout() {
                    Err(anyhow!("Server not available. Please check the server status and try again."))
                } else {
                    Err(anyhow!("Request to server failed: {}", err))
                }
            }
        }
    }

    #[async_recursion::async_recursion]
    pub async fn shell_execute(&self, text: &str) -> Result<()> {
        let spinner = create_spinner("Translating").await;
        let result = self.chat(text, false).await;
        spinner.stop();

        match result {
            Ok(answer) => {
                let eval_str = answer.result;

                if let Some(error) = answer.error {
                    eprintln!("Server error: {}", error.message);
                    return Err(anyhow!(error.message));
                }

                loop {
                    let answer = Select::new(
                        eval_str.trim(),
                        vec!["✅ Execute", "📖 Explain", "❌ Cancel"],
                    )
                        .prompt()?;

                    match answer {
                        "✅ Execute" => {
                            debug!("{} {:?}", SHELL.cmd, &[&SHELL.arg, &eval_str]);
                            let code = run_command(&SHELL.cmd, &[&SHELL.arg, &eval_str], None)?;
                            if code != 0 {
                                process::exit(code);
                            }
                        }
                        "📖 Explain" => {
                            let answer = self.chat(&eval_str, true).await?;
                            if let Some(error) = answer.error {
                                eprintln!("Error: {}", error.message);
                            } else {
                                println!("{}", answer.result);
                            }
                            continue;
                        }
                        _ => {}
                    }
                    break;
                }
            }
            Err(err) => {
                eprintln!("Error: {}", err);
                return Err(err);
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
