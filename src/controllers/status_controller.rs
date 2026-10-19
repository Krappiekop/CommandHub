use axum::Json;

use crate::models::status::StatusResponse;

pub async fn status() -> Json<StatusResponse> {
    Json(StatusResponse {
        status: "ok",
        applicatie: "commandhub",
    })
}
