use crate::config::{ConfigFile, ToolDefinition};

pub struct ToolRegistry {
    tools: Vec<ToolDefinition>,
}

impl ToolRegistry {
    pub fn new(config: ConfigFile) -> Self {
        Self {
            tools: config.tools,
        }
    }

    pub fn tools(&self) -> &[ToolDefinition] {
        &self.tools
    }
}
