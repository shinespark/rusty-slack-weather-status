use std::collections::HashMap;

use reqwest::{header, Client, StatusCode};

const SET_USERS_PROFILE_API: &str = "https://slack.com/api/users.profile.set";

pub struct SlackRequest {
    token: String,
}

impl SlackRequest {
    pub fn new(token: &str) -> Self {
        Self {
            token: format!("Bearer {token}"),
        }
    }

    pub async fn update_status(
        &self,
        emoji: &str,
        text: &str,
    ) -> Result<(StatusCode, String), Box<dyn std::error::Error>> {
        let mut profile = HashMap::new();
        profile.insert("status_emoji", emoji);
        profile.insert("status_text", text);
        let mut map = HashMap::new();
        map.insert("profile", profile);

        let client = Client::builder().build()?;
        let res = client
            .post(SET_USERS_PROFILE_API)
            .header(header::AUTHORIZATION, &self.token)
            .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
            .json(&map)
            .send()
            .await?;

        let status_code = res.status();
        let body = res.text().await?;
        Ok((status_code, body))
    }
}
