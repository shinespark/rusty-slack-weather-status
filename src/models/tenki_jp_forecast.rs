use std::path::Path;

use reqwest::StatusCode;
use scraper::{Html, Selector};

use crate::models::{Forecast, TempDiff};

const TRIM_CHARS: [char; 3] = ['[', '+', ']'];
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36";

#[derive(Debug)]
pub struct TenkiJpForecast {
    status: StatusCode,
    html: Html,
}

impl TenkiJpForecast {
    pub async fn get(url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = reqwest::Client::builder().user_agent(USER_AGENT).build()?;
        let res = client.get(url).send().await?;
        Ok(Self {
            status: res.status(),
            html: Html::parse_document(&res.text().await?),
        })
    }

    pub fn parse(&self) -> Result<Forecast, Box<dyn std::error::Error>> {
        Ok(Forecast {
            place: self.get_text("h2").split("の天気").collect::<Vec<_>>()[0].to_string(),
            date_time: self
                .get_text(".date-time")
                .split("発表")
                .collect::<Vec<_>>()[0]
                .to_string(),
            special_warnings: self.get_texts(".special-warn-entry"),
            warnings: self.get_texts(".warn-entry"),
            alerts: self.get_texts(".alert-entry"),
            weather: self.get_text(".weather-telop"),
            weather_icon_name: self.get_weather_icon_name(".weather-icon > img ", "src"),
            high_temp: self
                .get_text("dd.high-temp > .value")
                .parse::<i16>()
                .unwrap(),
            high_temp_diff: TempDiff::new(
                &self
                    .get_text("dd.high-temp.tempdiff")
                    .replace(TRIM_CHARS.as_ref(), ""),
            ),
            low_temp: self
                .get_text("dd.low-temp > .value")
                .parse::<i16>()
                .unwrap(),
            low_temp_diff: TempDiff::new(
                &self
                    .get_text("dd.low-temp.tempdiff")
                    .replace(TRIM_CHARS.as_ref(), ""),
            ),
        })
    }

    fn get_text(&self, selector: &str) -> String {
        let selector = Selector::parse(selector).unwrap();
        match self.html.select(&selector).next() {
            Some(x) => x
                .text()
                .collect::<Vec<_>>()
                .into_iter()
                .collect::<String>()
                .trim()
                .into(),
            None => "".to_string(),
        }
    }

    fn get_texts(&self, selector: &str) -> Option<Vec<String>> {
        let selector = Selector::parse(selector).unwrap();
        let texts = self
            .html
            .select(&selector)
            .map(|x| {
                x.text()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .collect::<String>()
                    .trim()
                    .into()
            })
            .collect::<Vec<_>>();

        match texts.len() {
            0 => None,
            _ => Some(texts),
        }
    }

    fn get_weather_icon_name(&self, selector: &str, attr: &str) -> String {
        let weather_icon_path = self.get_attr(selector, attr).unwrap_or_default();
        Self::get_file_stem(&weather_icon_path)
    }

    fn get_attr(&self, selector: &str, attr: &str) -> Option<String> {
        let selector = Selector::parse(selector).unwrap();

        self.html
            .select(&selector)
            .next()
            .unwrap() // ここで失敗した模様
            .value()
            .attr(attr)
            .map(|x| x.into())
    }

    fn get_file_stem(path: &str) -> String {
        Path::new(path)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_get_text() {
        let tenki_jp_forecast = TenkiJpForecast {
            status: Default::default(),
            html: Html::parse_document(
                "<html><h1>h1要素</h1><h2>h2要素</h2><h3>h3要素</h3></html>",
            ),
        };

        assert_eq!(tenki_jp_forecast.get_text("h2"), "h2要素");
    }

    mod test_get_texts {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn found() {
            let tenki_jp_forecast = TenkiJpForecast {
                status: Default::default(),
                html: Html::parse_document(
                    "<html><span class='alert-entry'>洪水</span><span class='alert-entry'>雷</span></html>",
                ),
            };

            assert_eq!(
                tenki_jp_forecast.get_texts(".alert-entry"),
                Some(vec!["洪水".to_string(), "雷".to_string()])
            );
        }

        #[test]
        fn not_found() {
            let tenki_jp_forecast = TenkiJpForecast {
                status: Default::default(),
                html: Html::parse_document("<html></html>"),
            };

            assert_eq!(tenki_jp_forecast.get_texts(".alert-entry"), None);
        }
    }

    #[test]
    fn test_get_attr() {
        let tenki_jp_forecast = TenkiJpForecast {
            status: Default::default(),
            html: Html::parse_document(
                "<html><img src='https://static.tenki.jp/images/icon/forecast-days-weather/12.png'></html>",
            ),
        };

        assert_eq!(
            tenki_jp_forecast.get_attr("img", "src"),
            Some("https://static.tenki.jp/images/icon/forecast-days-weather/12.png".to_string())
        );
    }

    mod test_get_file_stem {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn daytime() {
            let str = "https://static.tenki.jp/images/icon/forecast-days-weather/12.png";
            let result = TenkiJpForecast::get_file_stem(str);

            assert_eq!(result, "12".to_string());
        }

        #[test]
        fn night() {
            let str = "https://static.tenki.jp/images/icon/forecast-days-weather/12_n.png";
            let result = TenkiJpForecast::get_file_stem(str);

            assert_eq!(result, "12_n".to_string());
        }
    }
}
