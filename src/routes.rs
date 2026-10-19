use axum::{Router, routing::get};

use crate::controllers::status_controller::status;

pub fn router() -> Router {
    Router::new().route("/status", get(status))
}
