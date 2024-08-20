use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct Prompts {
    pub explain: String,
    os_prompt: String,
    combinator_powershell: String,
    combinator_default: String,
    additional_instructions: String,
}

impl Prompts {
    pub fn from_yaml(file_path: &str) -> Self {
        let config_content =
            fs::read_to_string(file_path).expect("Failed to read the prompts file");
        Prompts::from_yaml_content(&config_content)
    }

    pub fn from_yaml_content(config_content: &str) -> Self {
        serde_yaml::from_str(config_content).expect("Failed to parse the prompts file")
    }

    pub fn shell_prompt(&self, os: &str, shell: &str) -> String {
        let combinator = if shell == "powershell" {
            &self.combinator_powershell
        } else {
            &self.combinator_default
        };
        format!(
            "{}\n{}\n{}",
            self.os_prompt.replace("{os}", os).replace("{shell}", shell),
            combinator,
            self.additional_instructions
        )
    }
}
