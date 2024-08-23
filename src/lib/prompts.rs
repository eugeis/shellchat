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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_from_yaml() {
        let yaml_content = r#"
        explain: "Explain prompt"
        os_prompt: "Operating system prompt for {os} and {shell}"
        combinator_powershell: "PowerShell combinator"
        combinator_default: "Default combinator"
        additional_instructions: "Additional instructions"
        "#;

        let prompts = Prompts::from_yaml_content(yaml_content);
        assert_eq!(prompts.explain, "Explain prompt");
        assert_eq!(
            prompts.os_prompt,
            "Operating system prompt for {os} and {shell}"
        );
        assert_eq!(prompts.combinator_powershell, "PowerShell combinator");
        assert_eq!(prompts.combinator_default, "Default combinator");
        assert_eq!(prompts.additional_instructions, "Additional instructions");
    }

    #[test]
    fn test_shell_prompt() {
        let yaml_content = r#"
        explain: "Explain prompt"
        os_prompt: "Operating system prompt for {os} and {shell}"
        combinator_powershell: "PowerShell combinator"
        combinator_default: "Default combinator"
        additional_instructions: "Additional instructions"
        "#;

        let prompts = Prompts::from_yaml_content(yaml_content);

        let shell_prompt = prompts.shell_prompt("Windows", "powershell");
        assert!(shell_prompt.contains("Operating system prompt for Windows and powershell"));
        assert!(shell_prompt.contains("PowerShell combinator"));
        assert!(shell_prompt.contains("Additional instructions"));

        let shell_prompt = prompts.shell_prompt("Linux", "bash");
        assert!(shell_prompt.contains("Operating system prompt for Linux and bash"));
        assert!(shell_prompt.contains("Default combinator"));
        assert!(shell_prompt.contains("Additional instructions"));
    }
    #[test]
    fn test_prompt_from_yaml_file_not_found() {
        let result = std::panic::catch_unwind(|| Prompts::from_yaml("non_existent_file.yaml"));
        assert!(result.is_err());
    }
}
