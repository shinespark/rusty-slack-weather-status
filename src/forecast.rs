use std::fmt::{self, Display, Formatter};
use std::path::Path;

use reqwest::StatusCode;
use scraper::{Html, Selector};

use crate::embed::{ALERT_EMOJI_MAP, WEATHER_EMOJI_MAP};

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
    special_warnings: Option<Vec<String>>, // 特別警報
    warnings: Option<Vec<String>>,         // 警報
    alerts: Option<Vec<String>>,           // 注意報
    weather: String,
    weather_icon_name: String,
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
            .unwrap()
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

impl Forecast {
    pub fn build_emoji(&self) -> String {
        match self.has_alert_text() {
            Some(x) => self.build_alert_emoji(x),
            None => self.build_weather_emoji(),
        }
    }

    fn has_alert_text(&self) -> Option<&String> {
        if let Some(special_warnings) = &self.special_warnings {
            return special_warnings.first();
        }

        if let Some(warnings) = &self.warnings {
            return warnings.first();
        }

        if let Some(alerts) = &self.alerts {
            return alerts.first();
        }

        None
    }

    fn build_alert_emoji(&self, alert_text: &String) -> String {
        ALERT_EMOJI_MAP.get(alert_text).unwrap().to_string()
    }

    fn build_weather_emoji(&self) -> String {
        let weather_icon_num = self.weather_icon_name.replace("_n", "");
        WEATHER_EMOJI_MAP
            .get(&weather_icon_num)
            .unwrap()
            .to_string()
    }

    pub fn build_text(&self) -> String {
        let advisory_text = match self.special_warnings.is_some()
            || self.warnings.is_some()
            || self.alerts.is_some()
        {
            true => {
                let special_warnings = self.special_warnings.as_ref().map(|x| {
                    x.iter()
                        .map(|special_warning| format!("{}特別警報", special_warning))
                        .collect::<Vec<_>>()
                        .join(",")
                });

                let warnings = self.warnings.as_ref().map(|x| {
                    x.iter()
                        .map(|warning| format!("{}警報", warning))
                        .collect::<Vec<_>>()
                        .join(",")
                });

                let alerts = self.alerts.as_ref().map(|x| {
                    x.iter()
                        .map(|alert| format!("{}注意報", alert))
                        .collect::<Vec<_>>()
                        .join(",")
                });

                let texts = vec![special_warnings, warnings, alerts]
                    .into_iter()
                    .filter_map(|x| x)
                    .collect::<Vec<_>>()
                    .join(",");

                Some(texts)
            }
            false => None,
        };
        let weather_text = format!(
            "{w} 最高: {ht}℃[{htd}] 最低: {lt}℃[{ltd}] 発表: {dt}",
            w = self.weather,
            ht = self.high_temp,
            htd = self.high_temp_diff,
            lt = self.low_temp,
            ltd = self.low_temp_diff,
            dt = self.date_time
        );

        match advisory_text.is_some() {
            true => format!(
                "{}: {} {}: {}",
                self.place,
                advisory_text.unwrap(),
                self.build_weather_emoji(),
                weather_text
            ),
            false => format!("{}: {}", self.place, weather_text),
        }
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

    mod test_build_emoji {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn no_alerts() {
            let forecast = Forecast {
                place: "場所".to_string(),
                date_time: "日時".to_string(),
                special_warnings: None,
                warnings: None,
                alerts: None,
                weather: "晴".to_string(),
                weather_icon_name: "01".to_string(),
                high_temp: 10,
                high_temp_diff: TempDiff::new("3"),
                low_temp: 0,
                low_temp_diff: TempDiff::new("-5"),
            };

            assert_eq!(forecast.build_emoji(), ":sunny:");
        }

        #[test]
        fn special_warnings() {
            let forecast = Forecast {
                place: "場所".to_string(),
                date_time: "日時".to_string(),
                special_warnings: Some(vec!["大雨".to_string()]),
                warnings: Some(vec!["洪水".to_string()]),
                alerts: Some(vec!["強風".to_string(), "雷".to_string()]),
                weather: "晴".to_string(),
                weather_icon_name: "01".to_string(),
                high_temp: 10,
                high_temp_diff: TempDiff::new("3"),
                low_temp: 0,
                low_temp_diff: TempDiff::new("-5"),
            };

            assert_eq!(forecast.build_emoji(), ":bucket:");
        }

        #[test]
        fn warnings() {
            let forecast = Forecast {
                place: "場所".to_string(),
                date_time: "日時".to_string(),
                special_warnings: None,
                warnings: Some(vec!["大雨".to_string()]),
                alerts: Some(vec!["洪水".to_string(), "雷".to_string()]),
                weather: "晴".to_string(),
                weather_icon_name: "01".to_string(),
                high_temp: 10,
                high_temp_diff: TempDiff::new("3"),
                low_temp: 0,
                low_temp_diff: TempDiff::new("-5"),
            };

            assert_eq!(forecast.build_emoji(), ":bucket:");
        }

        #[test]
        fn alerts() {
            let forecast = Forecast {
                place: "場所".to_string(),
                date_time: "日時".to_string(),
                special_warnings: None,
                warnings: None,
                alerts: Some(vec!["洪水".to_string(), "雷".to_string()]),
                weather: "晴".to_string(),
                weather_icon_name: "01".to_string(),
                high_temp: 10,
                high_temp_diff: TempDiff::new("3"),
                low_temp: 0,
                low_temp_diff: TempDiff::new("-5"),
            };

            assert_eq!(forecast.build_emoji(), ":ocean:");
        }
    }

    mod test_build_text {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn no_alerts() {
            let forecast = Forecast {
                place: "場所".to_string(),
                date_time: "日時".to_string(),
                special_warnings: None,
                warnings: None,
                alerts: None,
                weather: "晴".to_string(),
                weather_icon_name: "01".to_string(),
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

        #[test]
        fn special_warnings() {
            let forecast = Forecast {
                place: "場所".to_string(),
                date_time: "日時".to_string(),
                special_warnings: Some(vec!["大雨".to_string()]),
                warnings: Some(vec!["洪水".to_string()]),
                alerts: Some(vec!["強風".to_string(), "雷".to_string()]),
                weather: "晴".to_string(),
                weather_icon_name: "01".to_string(),
                high_temp: 10,
                high_temp_diff: TempDiff::new("3"),
                low_temp: 0,
                low_temp_diff: TempDiff::new("-5"),
            };

            assert_eq!(
                forecast.build_text(),
                "場所: 大雨特別警報,洪水警報,強風注意報,雷注意報 :sunny:: 晴 最高: 10℃[+3] 最低: 0℃[-5] 発表: 日時"
            );
        }

        #[test]
        fn warnings() {
            let forecast = Forecast {
                place: "場所".to_string(),
                date_time: "日時".to_string(),
                special_warnings: None,
                warnings: Some(vec!["大雨".to_string()]),
                alerts: None,
                weather: "晴".to_string(),
                weather_icon_name: "01".to_string(),
                high_temp: 10,
                high_temp_diff: TempDiff::new("3"),
                low_temp: 0,
                low_temp_diff: TempDiff::new("-5"),
            };

            assert_eq!(
                forecast.build_text(),
                "場所: 大雨警報 :sunny:: 晴 最高: 10℃[+3] 最低: 0℃[-5] 発表: 日時"
            );
        }

        #[test]
        fn alerts() {
            let forecast = Forecast {
                place: "場所".to_string(),
                date_time: "日時".to_string(),
                special_warnings: None,
                warnings: Some(vec!["大雨".to_string()]),
                alerts: Some(vec!["洪水".to_string(), "雷".to_string()]),
                weather: "晴".to_string(),
                weather_icon_name: "01".to_string(),
                high_temp: 10,
                high_temp_diff: TempDiff::new("3"),
                low_temp: 0,
                low_temp_diff: TempDiff::new("-5"),
            };

            assert_eq!(
                forecast.build_text(),
                "場所: 大雨警報,洪水注意報,雷注意報 :sunny:: 晴 最高: 10℃[+3] 最低: 0℃[-5] 発表: 日時"
            );
        }
    }
}
