use std::collections::HashMap;
use std::path::Path;

use reqwest::{get, header, Client, StatusCode};
use scraper::{Html, Selector};

const TRIM_CHARS: [char; 3] = ['[', '+', ']'];
const GET_USERS_PROFILE_API: &str = "https://slack.com/api/users.profile.get";
const SET_USERS_PROFILE_API: &str = "https://slack.com/api/users.profile.set";

// TODO: impl でコンストラクタとして生成するようにしてもいいかも
#[derive(Debug)]
pub struct Forecast {
    place: String,
    date_time: String,
    weather: String,
    weather_icon: String,
    high_temp: i16,
    high_temp_diff: i16,
    low_temp: i16,
    low_temp_diff: i16,
}

pub async fn get_forecast(url: &str) -> Result<Option<Forecast>, Box<dyn std::error::Error>> {
    let res = get(url).await?;
    match res.status().is_success() {
        true => Ok(parse_forecast(&*res.text().await?).into()),
        false => Ok(None),
    }
}

fn parse_forecast(doc: &str) -> Forecast {
    let html = Html::parse_document(doc);

    Forecast {
        place: get_text(&html, "h2").split("の天気").collect::<Vec<_>>()[0].to_string(),
        date_time: get_text(&html, ".date-time")
            .split("発表")
            .collect::<Vec<_>>()[0]
            .to_string(),
        weather: get_text(&html, ".weather-telop"),
        weather_icon: get_stem(&get_attr(&html, ".weather-icon > img ", "src").unwrap_or_default()),
        high_temp: get_text(&html, "dd.high-temp > .value")
            .parse::<i16>()
            .unwrap(),
        high_temp_diff: get_text(&html, "dd.high-temp.tempdiff")
            .replace(TRIM_CHARS.as_ref(), "")
            .parse::<i16>()
            .unwrap(),
        low_temp: get_text(&html, "dd.low-temp > .value")
            .parse::<i16>()
            .unwrap(),
        low_temp_diff: get_text(&html, "dd.low-temp.tempdiff")
            .replace(TRIM_CHARS.as_ref(), "")
            .parse::<i16>()
            .unwrap(),
    }
}

fn get_text(html: &Html, selectors: &str) -> String {
    let selector = Selector::parse(selectors).unwrap();
    html.select(&selector)
        .next()
        .unwrap()
        .text()
        .collect::<Vec<_>>()
        .into_iter()
        .collect::<String>()
        .trim()
        .into()
}

fn get_attr(html: &Html, selectors: &str, attr: &str) -> Option<String> {
    let selector = Selector::parse(selectors).unwrap();
    html.select(&selector)
        .next()
        .unwrap()
        .value()
        .attr(attr)
        .map(|x| x.into())
}

fn get_stem(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_get_text() {
        let html =
            Html::parse_document("<html><h1>h1要素</h1><h2>h2要素</h2><h3>h3要素</h3></html>");
        let result = get_text(&html, "h2");

        assert_eq!(result, "h2要素");
    }

    #[test]
    fn test_get_attr() {
        let html = Html::parse_document("<html><img src='https://static.tenki.jp/images/icon/forecast-days-weather/12.png'></html>");
        let result = get_attr(&html, "img", "src");

        assert_eq!(
            result,
            Some("https://static.tenki.jp/images/icon/forecast-days-weather/12.png".to_string())
        );
    }

    #[test]
    fn test_get_stem() {
        let str = "https://static.tenki.jp/images/icon/forecast-days-weather/12.png";
        let result = get_stem(str);

        assert_eq!(result, "12".to_string());

        let str = "https://static.tenki.jp/images/icon/forecast-days-weather/12_n.png";
        let result = get_stem(str);

        assert_eq!(result, "12_n".to_string());
    }
}
