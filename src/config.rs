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

            // Controleer of de timeout, indien aanwezig, binnen het bereik van 1 tot 60000 ms ligt
            if let Some(timeout) = tool.timeout
                && !(1..=60_000).contains(&timeout)
            {
                return Err(format!(
                    "Timeout voor tool '{}' moet tussen 1 en 60000 ms liggen",
                    tool.name
                ));
            }

            let mut parameter_names: HashSet<&str> = HashSet::new();

            // Controleer op dubbele parameter namen
            for parameter in &tool.parameters {
                if !parameter_names.insert(parameter.name.as_str()) {
                    return Err(format!("Dubbele parameter in tool '{}'", tool.name));
                }
            }

            // Controleer of alle placeholders in de argumenten verwijzen naar bestaande parameters
            for argument in &tool.arguments {
                if !argument.contains('{') && !argument.contains('}') {
                    continue;
                }

                // Controleer of de placeholder een volledig argument is, d.w.z. het moet beginnen met '{' en eindigen met '}'
                let Some(after_open_brace) = argument.strip_prefix('{') else {
                    return Err(format!(
                        "Placeholder moet een volledig argument zijn in tool '{}'",
                        tool.name
                    ));
                };

                // Controleer of de placeholder eindigt met '}'
                let Some(parameter_name) = after_open_brace.strip_suffix('}') else {
                    return Err(format!(
                        "Placeholder moet een volledig argument zijn in tool '{}'",
                        tool.name
                    ));
                };

                // Controleer of de placeholder geen lege naam heeft en geen extra '{' of '}' bevat
                if parameter_name.is_empty()
                    || parameter_name.contains('{')
                    || parameter_name.contains('}')
                {
                    return Err(format!("Ongeldige placeholder in tool '{}'", tool.name));
                }

                // Controleer of de placeholder verwijst naar een bestaande parameter
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
// De definitie van een tool
pub struct ToolDefinition {
    pub name: String,
    pub description: String,

    #[expect(dead_code, reason = "Gebruikt door de procesuitvoerder in F03")]
    pub command: String,

    pub arguments: Vec<String>,
    pub parameters: Vec<ParameterDefinition>,
    pub timeout: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
// De parameterdefinitie voor een tool
pub struct ParameterDefinition {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: ParameterType,
    pub required: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
// De parameter types die momenteel worden ondersteund
pub enum ParameterType {
    String,
}

/// Laad de configuratie uit het bestand dat wordt opgegeven door de omgevingsvariabele COMMANDHUB_CONFIG,
pub fn load() -> Result<ConfigFile, std::io::Error> {
    let path = std::env::var_os("COMMANDHUB_CONFIG")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("config/tools.json"));

    load_from_path(&path)
}

/// Laad de configuratie uit het opgegeven pad
fn load_from_path(path: &std::path::Path) -> Result<ConfigFile, std::io::Error> {
    let contents = std::fs::read_to_string(path).map_err(|error| {
        std::io::Error::new(
            error.kind(),
            format!(
                "Kan configuratiebestand {} niet lezen: {error}",
                path.display()
            ),
        )
    })?;

    // Parse de JSON en valideer de configuratie
    let config: ConfigFile = serde_json::from_str(&contents).map_err(|error| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "Ongeldige JSON in configuratiebestand {}: {error}",
                path.display()
            ),
        )
    })?;

    // Valideer de configuratie
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

// ------------------------------------------ TESTS ------------------------------------------

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
    // Test dat het ontbreken van de timeout resulteert in None
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
    // Test dat onbekende velden in de tooldefinitie worden afgewezen
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
    // Test dat dubbele toolnamen worden afgewezen
    fn duplicate_tool_names_are_rejected() {
        let config = ConfigFile {
            tools: vec![test_tool("duplicate", None), test_tool("duplicate", None)],
        };

        assert!(config.validate().is_err());
    }

    #[test]
    // Test dat timeouts buiten het toegestane bereik worden afgewezen
    fn timeout_outside_allowed_range_is_rejected() {
        for timeout in [Some(0), Some(60_001)] {
            let config = ConfigFile {
                tools: vec![test_tool("test_tool", timeout)],
            };

            assert!(config.validate().is_err());
        }
    }

    #[test]
    // Test dat een geldige volledige placeholder wordt geaccepteerd
    fn valid_full_placeholder_is_accepted() {
        let mut tool = test_tool("list_files", None);
        tool.arguments = vec!["{path}".to_owned()];
        tool.parameters = vec![test_parameter("path")];

        let config = ConfigFile { tools: vec![tool] };

        assert!(config.validate().is_ok());
    }

    #[test]
    // Test dat een onbekende placeholder wordt afgewezen
    fn unknown_placeholder_is_rejected() {
        let mut tool = test_tool("list_files", None);
        tool.arguments = vec!["{missing}".to_owned()];
        tool.parameters = vec![test_parameter("path")];

        let config = ConfigFile { tools: vec![tool] };

        assert!(config.validate().is_err());
    }

    #[test]
    // Test dat een gedeeltelijke placeholder wordt afgewezen
    fn partial_placeholder_is_rejected() {
        let mut tool = test_tool("list_files", None);
        tool.arguments = vec!["prefix-{path}".to_owned()];
        tool.parameters = vec![test_parameter("path")];

        let config = ConfigFile { tools: vec![tool] };

        assert!(config.validate().is_err());
    }

    #[test]
    // Test dat dubbele parameter namen worden afgewezen
    fn duplicate_parameter_names_are_rejected() {
        let mut tool = test_tool("list_files", None);
        tool.parameters = vec![test_parameter("path"), test_parameter("path")];

        let config = ConfigFile { tools: vec![tool] };

        assert!(config.validate().is_err());
    }

    #[test]
    // Test dat ongeldige JSON wordt afgewezen
    fn invalid_json_is_rejected() {
        let result = serde_json::from_str::<ConfigFile>(r#"{"tools":["#);

        assert!(result.is_err());
    }

    #[test]
    // Test dat ontbrekende verplichte velden in de tooldefinitie worden afgewezen
    fn missing_required_tool_field_is_rejected() {
        let json = r#"
    {
        "tools": [
            {
                "name": "git_version",
                "description": "Show the installed Git version",
                "arguments": [],
                "parameters": []
            }
        ]
    }
    "#;

        let result = serde_json::from_str::<ConfigFile>(json);

        assert!(result.is_err());
    }

    #[test]
    // Test dat het ontbreken van het configuratiebestand resulteert in een NotFound-fout
    fn missing_config_file_returns_not_found() {
        let path =
            std::env::temp_dir().join(format!("commandhub-missing-config-{}", std::process::id()));

        let error = load_from_path(&path).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
    }

    #[test]
    // Test dat een ongeldige JSON in het configuratiebestand resulteert in een InvalidData-fout
    fn invalid_json_file_returns_invalid_data() {
        let path = std::env::temp_dir().join(format!(
            "commandhub-invalid-json-{}.json",
            std::process::id()
        ));

        std::fs::write(&path, r#"{"tools:["#).expect("testbestand moet geschreven kunnen worden");

        let error = load_from_path(&path).expect_err("kapotte JSON moet worden afgewezen");

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("Ongeldige JSON"));

        std::fs::remove_file(path).expect("testbestand moet opgeruimd kunnen worden");
    }

    #[test]
    // Test dat een configuratiebestand met dubbele toolnamen resulteert in een InvalidData-fout
    fn invalid_config_file_returns_invalid_data() {
        let path = std::env::temp_dir().join(format!(
            "commandhub-invalid-config-{}.json",
            std::process::id()
        ));

        let json = r#"{"tools":[
            {"name":"duplicate","description":"First","command":"test","arguments":[],"parameters":[]},
            {"name":"duplicate","description":"Second","command":"test","arguments":[],"parameters":[]}
        ]}"#;

        std::fs::write(&path, json).expect("testbestand moet geschreven kunnen worden");

        let error = load_from_path(&path).expect_err("dubbele toolnamen moeten worden afgewezen");

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("Ongeldige configuratie"));

        std::fs::remove_file(path).expect("testbestand moet opgeruimd kunnen worden");
    }
}
