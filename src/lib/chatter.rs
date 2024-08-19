use std::process;
use inquire::{Select, Text};
use log::debug;
use reqwest::Client;
use crate::command;
use crate::command::SHELL;
use crate::common;
use crate::common::ShellRequest;
use crate::common::ShellResponse;
use crate::common::HEADER_API_KEY;
use crate::spinner::create_spinner;
use crate::command::run_command;

#[derive(Debug)]
pub struct Chatter {
    endpoint: String,
    shell: String,
    os: String,
    client: Client,
    api_key: String,
}

impl Chatter {
    pub fn new(endpoint: &str, api_key: &str) -> Self {
        Chatter {
            endpoint: endpoint.to_string(),
            shell: SHELL.name.clone(),
            os: String::from(command::OS.as_str()), // Using OS directly
            client: Client::new(),
            api_key: api_key.to_string(),
        }
    }

    fn build_request(&self, prompt: &str, explain: bool) -> ShellRequest {
        ShellRequest {
            os: self.os.clone(),
            shell: self.shell.clone(),
            prompt: prompt.to_string(),
            explain,
        }
    }

    async fn chat(&self, prompt: &str, explain: bool) -> ShellResponse {
        let res = self.client
            .post(&self.endpoint)
            .header(HEADER_API_KEY, &self.api_key)
            .json(&self.build_request(prompt, explain))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        return res
    }

    #[async_recursion::async_recursion]
    pub async fn shell_execute(&self, text: &str) -> anyhow::Result<()> {
        let spinner = create_spinner("Generating").await;
        let response = self.chat(text, false).await;
        let eval_str = response.result;
        spinner.stop();

        loop {
            let answer = Select::new(
                eval_str.trim(),
                vec!["✅ Execute", "🔄️ Revise", "📖 Explain", "❌ Cancel"],
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
                "🔄️ Revise" => {
                    let revision = Text::new("Enter your revision:").prompt()?;
                    let text = format!("{}\n{revision}", eval_str);
                    return self.shell_execute(&text).await;
                }
                "📖 Explain" => {
                    let answer = self.chat(&eval_str, true).await;
                    println!("{}", answer.result);
                    continue;
                }
                _ => {}
            }
            break;
        }
        Ok(())
    }
}