use std::str::FromStr;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub listen: String,
}

impl Settings {
    pub fn load() -> color_eyre::Result<Self> {
        // TODO: don't hardcode this
        let settings = include_str!("../data/settings.ron");
        let settings = ron::from_str(settings)?;

        Ok(settings)
    }
}