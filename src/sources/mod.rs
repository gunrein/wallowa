use serde::{Deserialize, Serialize};

pub mod github;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Github,
}