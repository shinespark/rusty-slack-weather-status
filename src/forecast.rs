use std::path::Path;

use reqwest;
use reqwest::StatusCode;
use scraper::{Html, Selector};

const TRIM_CHARS: [char; 3] = ['[', '+', ']'];

#[derive(Debug)]
pub struct TenkiJpForecast {
    status: StatusCode,
    html: Html,
}

#[derive(Debug)]
pub struct Forecast {
    place: String,
    date_time: String,
    pub weather: String,
    weather_icon_stem: String,
    high_temp: i16,
    high_temp_diff: i16,
    low_temp: i16,
    low_temp_diff: i16,
}

impl TenkiJpForecast {
    pub async fn get(url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let res = reqwest::get(url).await?;
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
            weather: self.get_text(".weather-telop"),
            weather_icon_stem: self.get_weather_icon_stem(".weather-icon > img ", "src"),
            high_temp: self
                .get_text("dd.high-temp > .value")
                .parse::<i16>()
                .unwrap(),
            high_temp_diff: self
                .get_text("dd.high-temp.tempdiff")
                .replace(TRIM_CHARS.as_ref(), "")
                .parse::<i16>()
                .unwrap(),
            low_temp: self
                .get_text("dd.low-temp > .value")
                .parse::<i16>()
                .unwrap(),
            low_temp_diff: self
                .get_text("dd.low-temp.tempdiff")
                .replace(TRIM_CHARS.as_ref(), "")
                .parse::<i16>()
                .unwrap(),
        })
    }

    fn get_text(&self, selector: &str) -> String {
        let selector = Selector::parse(selector).unwrap();

        self.html
            .select(&selector)
            .next()
            .unwrap()
            .text()
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<String>()
            .trim()
            .into()
    }

    fn get_weather_icon_stem(&self, selector: &str, attr: &str) -> String {
        let weather_icon_path = self.get_attr(selector, attr).unwrap_or_default();
        Self::get_stem(&weather_icon_path)
    }

    fn get_attr(&self, selector: &str, attr: &str) -> Option<String> {
        let selector = Selector::parse(selector).unwrap();

        self.html
            .select(&selector)
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

    #[test]
    fn test_get_stem() {
        let str = "https://static.tenki.jp/images/icon/forecast-days-weather/12.png";
        let result = TenkiJpForecast::get_stem(str);

        assert_eq!(result, "12".to_string());

        let str = "https://static.tenki.jp/images/icon/forecast-days-weather/12_n.png";
        let result = TenkiJpForecast::get_stem(str);

        assert_eq!(result, "12_n".to_string());
    }
}
