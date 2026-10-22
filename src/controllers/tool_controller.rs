use std::sync::Arc;

use axum::{Json, extract::State};
use serde_json::{Map, Value, json};

use crate::{config::ParameterType, registry::ToolRegistry};

pub async fn list_tools(State(registry): State<Arc<ToolRegistry>>) -> Json<Value> {
    let tools = registry
        .tools()
        .iter()
        .map(|tool| {
            let properties = tool
                .parameters
                .iter()
                .map(|parameter| {
                    let parameter_type = match &parameter.kind {
                        ParameterType::String => "string",
                    };

                    (parameter.name.clone(), json!({ "type": parameter_type }))
                })
                .collect::<Map<String, Value>>();

            let required = tool
                .parameters
                .iter()
                .filter(|parameter| parameter.required)
                .map(|parameter| parameter.name.clone())
                .collect::<Vec<_>>();

            json!({
                "name": tool.name,
                "description": tool.description,
                "inputSchema": {
                    "type": "object",
                    "properties": properties,
                    "required": required
                }
            })
        })
        .collect::<Vec<_>>();

    Json(json!({ "tools": tools }))
}
