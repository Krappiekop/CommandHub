use serde::Serialize;

#[derive(Serialize)]
pub struct StatusResponse {
    pub status: &'static str,
    pub applicatie: &'static str,
}
