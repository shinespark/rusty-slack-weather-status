use std::collections::HashMap;

use reqwest::{header, Client, StatusCode};

const GET_USERS_PROFILE_API: &str = "https://slack.com/api/users.profile.get";
const SET_USERS_PROFILE_API: &str = "https://slack.com/api/users.profile.set";

struct JsonBody {
    profile: Profile,
}

struct Profile {
    status_emoji: String,
    status_text: String,
}

pub async fn get_slack_status(
    token: &str,
) -> Result<(StatusCode, String), Box<dyn std::error::Error>> {
    let bearer_token = format!("Bearer {}", token);
    let client = Client::builder().build()?;
    let res = client
        .get(GET_USERS_PROFILE_API)
        .header(header::AUTHORIZATION, bearer_token)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .send()
        .await?;

    let status = res.status();
    let body = res.text().await?;
    Ok((status, body))
}

pub async fn update_slack_status(
    token: &str,
    emoji: &str,
    text: &str,
) -> Result<(StatusCode, String), Box<dyn std::error::Error>> {
    let bearer_token = format!("Bearer {}", token);
    let mut profile = HashMap::new();
    profile.insert("status_emoji", emoji);
    profile.insert("status_text", text);
    let mut map = HashMap::new();
    map.insert("profile", profile);

    let client = Client::builder().build()?;
    let res = client
        .post(SET_USERS_PROFILE_API)
        .header(header::AUTHORIZATION, bearer_token)
        .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
        .json(&map)
        .send()
        .await?;

    let status = res.status();
    let body = res.text().await?;
    Ok((status, body))
}
