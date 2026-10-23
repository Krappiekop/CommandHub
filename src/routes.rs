use std::sync::Arc;

use axum::{Router, routing::get};

use crate::{
    controllers::{status_controller::status, tool_controller::list_tools},
    registry::ToolRegistry,
};

pub fn router(registry: Arc<ToolRegistry>) -> Router {
    Router::new()
        .route("/status", get(status))
        .route("/tools", get(list_tools))
        .with_state(registry)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use serde_json::json;
    use tower::ServiceExt;

    use crate::{
        config::{ConfigFile, ParameterDefinition, ParameterType, ToolDefinition},
        registry::ToolRegistry,
    };

    use super::router;

    fn test_registry() -> Arc<ToolRegistry> {
        let config = ConfigFile {
            tools: vec![ToolDefinition {
                name: "list_files".to_owned(),
                description: "List files in a directory".to_owned(),
                command: "private-command".to_owned(),
                arguments: vec!["{path}".to_owned()],
                parameters: vec![ParameterDefinition {
                    name: "path".to_owned(),
                    kind: ParameterType::String,
                    required: true,
                }],
                timeout: None,
            }],
        };

        Arc::new(ToolRegistry::new(config))
    }

    #[tokio::test]
    async fn tools_route_returns_tool_metadata_and_input_schema() {
        let response = router(test_registry())
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/tools")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        let actual: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let expected = json!({
            "tools": [
                {
                    "name": "list_files",
                    "description": "List files in a directory",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string"
                            }
                        },
                        "required": ["path"]
                    }
                }
            ]
        });

        assert_eq!(actual, expected);
    }
}
