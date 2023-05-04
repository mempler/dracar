use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ServiceScript {
    Exec{ command: String, arguments: Vec<String> },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Service {
    pub description: String,
    pub scripts: ServiceScripts
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceScripts {
    pub start: Option<ServiceScript>,
    pub stop: Option<ServiceScript>, // default: SIGTERM or SIGKILL if timeout is reached
}
