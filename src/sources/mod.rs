use anyhow::Result;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::{config_value, db::Pool};

use self::github::{fetch_pulls, request_pulls};

pub mod github;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Github,
}

pub async fn fetch_given_source(pool: &Pool, source_id: &Source) -> Result<NaiveDateTime> {
    match source_id {
        crate::sources::Source::Github => {
            let repos: Vec<String> = config_value("github.repos")
                .await
                .expect("Unable to get config for `github.repos`");
            let responses = request_pulls(pool, &repos).await?;
            fetch_pulls(pool, &responses)
        }
    }
}
