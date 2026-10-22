use serde::Deserialize;

use std::collections::HashSet;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigFile {
    pub tools: Vec<ToolDefinition>,
}

impl ConfigFile {
    pub fn validate(&self) -> Result<(), String> {
        let mut names = HashSet::new();

        for tool in &self.tools {
            if !names.insert(tool.name.as_str()) {
                return Err(format!("Dubbele toolnaam: {}", tool.name));
            }

            if let Some(timeout) = tool.timeout
                && !(1..=60_000).contains(&timeout)
            {
                return Err(format!(
                    "Timeout voor tool '{}' moet tussen 1 en 60000 ms liggen",
                    tool.name
                ));
            }

            let mut parameter_names: HashSet<&str> = HashSet::new();

            for parameter in &tool.parameters {
                if !parameter_names.insert(parameter.name.as_str()) {
                    return Err(format!("Dubbele parameter in tool '{}'", tool.name));
                }
            }

            for argument in &tool.arguments {
                if !argument.contains('{') && !argument.contains('}') {
                    continue;
                }

                let Some(after_open_brace) = argument.strip_prefix('{') else {
                    return Err(format!(
                        "Placeholder moet een volledig argument zijn in tool '{}'",
                        tool.name
                    ));
                };

                let Some(parameter_name) = after_open_brace.strip_suffix('}') else {
                    return Err(format!(
                        "Placeholder moet een volledig argument zijn in tool '{}'",
                        tool.name
                    ));
                };

                if parameter_name.is_empty()
                    || parameter_name.contains('{')
                    || parameter_name.contains('}')
                {
                    return Err(format!("Ongeldige placeholder in tool '{}'", tool.name));
                }

                if !parameter_names.contains(parameter_name) {
                    return Err(format!(
                        "Placeholder verwijst naar een onbekende parameter in tool '{}'",
                        tool.name
                    ));
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolDefinition {
    pub name: String,

    #[expect(dead_code, reason = "Gebruikt bij het toevoegen van GET /tools")]
    pub description: String,

    #[expect(dead_code, reason = "Gebruikt door de procesuitvoerder in F03")]
    pub command: String,

    pub arguments: Vec<String>,
    pub parameters: Vec<ParameterDefinition>,
    pub timeout: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParameterDefinition {
    pub name: String,

    #[serde(rename = "type")]
    #[expect(dead_code, reason = "Gebruikt bij het publiceren van het inputschema")]
    pub kind: ParameterType,

    #[expect(dead_code, reason = "Gebruikt bij het publiceren van het inputschema")]
    pub required: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterType {
    String,
}

pub fn load() -> Result<ConfigFile, std::io::Error> {
    let path = std::env::var_os("COMMANDHUB_CONFIG")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("config/tools.json"));

    let contents = std::fs::read_to_string(&path).map_err(|error| {
        std::io::Error::new(
            error.kind(),
            format!(
                "Kan configuratiebestand {} niet lezen: {error}",
                path.display()
            ),
        )
    })?;

    let config: ConfigFile = serde_json::from_str(&contents).map_err(|error| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "Ongeldige JSON in configuratiebestand {}: {error}",
                path.display()
            ),
        )
    })?;

    config.validate().map_err(|error| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "Ongeldige configuratie in bestand {}: {error}",
                path.display()
            ),
        )
    })?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_tool(name: &str, timeout: Option<u64>) -> ToolDefinition {
        ToolDefinition {
            name: name.to_owned(),
            description: "Test tool".to_owned(),
            command: "test-command".to_owned(),
            arguments: Vec::new(),
            parameters: Vec::new(),
            timeout,
        }
    }

    fn test_parameter(name: &str) -> ParameterDefinition {
        ParameterDefinition {
            name: name.to_owned(),
            kind: ParameterType::String,
            required: true,
        }
    }

    #[test]
    fn missing_timeout_becomes_none() {
        let json = r#"
        {
            "tools": [
                {
                    "name": "git_version",
                    "description": "Show the installed Git version",
                    "command": "git",
                    "arguments": ["--version"],
                    "parameters": []
                }
            ]
        }
        "#;

        let config: ConfigFile =
            serde_json::from_str(json).expect("geldige configuratie moet worden ingelezen");

        assert_eq!(config.tools[0].timeout, None);
    }

    #[test]
    fn unknown_tool_field_is_rejected() {
        let json = r#"
        {
            "tools": [
                {
                    "name": "git_version",
                    "description": "Show the installed Git version",
                    "command": "git",
                    "arguments": ["--version"],
                    "parameters": [],
                    "unexpected": true
                }
            ]
        }
        "#;

        assert!(serde_json::from_str::<ConfigFile>(json).is_err());
    }

    #[test]
    fn duplicate_tool_names_are_rejected() {
        let config = ConfigFile {
            tools: vec![test_tool("duplicate", None), test_tool("duplicate", None)],
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn timeout_outside_allowed_range_is_rejected() {
        for timeout in [Some(0), Some(60_001)] {
            let config = ConfigFile {
                tools: vec![test_tool("test_tool", timeout)],
            };

            assert!(config.validate().is_err());
        }
    }

    #[test]
    fn valid_full_placeholder_is_accepted() {
        let mut tool = test_tool("list_files", None);
        tool.arguments = vec!["{path}".to_owned()];
        tool.parameters = vec![test_parameter("path")];

        let config = ConfigFile { tools: vec![tool] };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn unknown_placeholder_is_rejected() {
        let mut tool = test_tool("list_files", None);
        tool.arguments = vec!["{missing}".to_owned()];
        tool.parameters = vec![test_parameter("path")];

        let config = ConfigFile { tools: vec![tool] };

        assert!(config.validate().is_err());
    }

    #[test]
    fn partial_placeholder_is_rejected() {
        let mut tool = test_tool("list_files", None);
        tool.arguments = vec!["prefix-{path}".to_owned()];
        tool.parameters = vec![test_parameter("path")];

        let config = ConfigFile { tools: vec![tool] };

        assert!(config.validate().is_err());
    }

    #[test]
    fn duplicate_parameter_names_are_rejected() {
        let mut tool = test_tool("list_files", None);
        tool.parameters = vec![test_parameter("path"), test_parameter("path")];

        let config = ConfigFile { tools: vec![tool] };

        assert!(config.validate().is_err());
    }
}
