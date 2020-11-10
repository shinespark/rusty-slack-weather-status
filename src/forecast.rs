use std::fmt::{self, Display, Formatter};
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
    weather: String,
    weather_icon_stem: String,
    high_temp: i16,
    high_temp_diff: TempDiff,
    low_temp: i16,
    low_temp_diff: TempDiff,
}

#[derive(Debug)]
struct TempDiff {
    temp_diff: i16,
}

impl Display for TempDiff {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.temp_diff.is_positive() {
            true => write!(f, "+{}", self.temp_diff),
            false => write!(f, "{}", self.temp_diff),
        }
    }
}

impl TempDiff {
    pub fn new(temp_diff: &str) -> Self {
        Self {
            temp_diff: temp_diff.parse::<i16>().unwrap(),
        }
    }
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

impl Forecast {
    pub fn build_text(&self) -> String {
        format!(
            "{}: {} 最高: {}℃[{}] 最低: {}℃[{}] 発表: {}",
            self.place,
            self.weather,
            self.high_temp,
            self.high_temp_diff,
            self.low_temp,
            self.low_temp_diff,
            self.date_time
        )
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

    #[test]
    fn test_build_text() {
        let forecast = Forecast {
            place: "場所".to_string(),
            date_time: "日時".to_string(),
            weather: "晴".to_string(),
            weather_icon_stem: "01".to_string(),
            high_temp: 10,
            high_temp_diff: TempDiff::new("3"),
            low_temp: 0,
            low_temp_diff: TempDiff::new("-5"),
        };

        assert_eq!(
            forecast.build_text(),
            "場所: 晴 最高: 10℃[+3] 最低: 0℃[-5] 発表: 日時"
        );
    }
}
