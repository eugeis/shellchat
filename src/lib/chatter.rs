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

    fn build_request(&self, prompt: &str, explain: bool) -> Question {
        Question {
            os: self.os.clone(),
            shell: self.shell.clone(),
            prompt: prompt.to_string(),
            explain,
        }
    }

    async fn chat(&self, prompt: &str, explain: bool) -> Answer {
        self.client
            .post(&self.url)
            .header(HEADER_API_KEY, &self.api_key)
            .json(&self.build_request(prompt, explain))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }

    #[async_recursion::async_recursion]
    pub async fn shell_execute(&self, text: &str) -> anyhow::Result<()> {
        let spinner = create_spinner("Translating").await;
        let response = self.chat(text, false).await;
        let eval_str = response.result;
        spinner.stop();

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
