use crate::embed::{ALERT_EMOJI_MAP, WEATHER_EMOJI_MAP};
use crate::models::temp_diff::TempDiff;

#[derive(Debug)]
pub struct Forecast {
    pub place: String,
    pub date_time: String,
    pub special_warnings: Option<Vec<String>>, // 特別警報
    pub warnings: Option<Vec<String>>,         // 警報
    pub alerts: Option<Vec<String>>,           // 注意報
    pub weather: String,
    pub weather_icon_name: String,
    pub high_temp: i16,
    pub high_temp_diff: TempDiff,
    pub low_temp: i16,
    pub low_temp_diff: TempDiff,
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
