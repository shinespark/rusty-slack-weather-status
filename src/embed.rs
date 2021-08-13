use std::collections::HashMap;

use lazy_static::lazy_static;
use rust_embed::RustEmbed;
use std::str::from_utf8;

const ALERT_EMOJI_MAP_TOML: &str = "alert_emoji_map.toml";
const WEATHER_EMOJI_MAP_TOML: &str = "weather_emoji_map.toml";

#[derive(RustEmbed)]
#[folder = "embed/"]
struct Embed;

lazy_static! {
    pub static ref ALERT_EMOJI_MAP: HashMap<String, String> = init_alert_emoji_map();
    pub static ref WEATHER_EMOJI_MAP: HashMap<String, String> = init_weather_emoji_map();
}

fn init_alert_emoji_map() -> HashMap<String, String> {
    let file = Embed::get(ALERT_EMOJI_MAP_TOML).expect("alert_emoji_map not found.");
    let raw = from_utf8(file.as_ref()).expect("alert_emoji_map couldn't open.");
    toml::from_str(raw).expect("alert_emoji_map couldn't parse.")
}

fn init_weather_emoji_map() -> HashMap<String, String> {
    let file = Embed::get(WEATHER_EMOJI_MAP_TOML).expect("weather_emoji_map not found.");
    let raw = from_utf8(file.as_ref()).expect("weather_emoji_map couldn't open.");
    toml::from_str(raw).expect("weather_emoji_map couldn't parse.")
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_alert_emoji_map() {
        assert_eq!(ALERT_EMOJI_MAP.get("暴風").unwrap(), ":cyclone:");
        assert_eq!(ALERT_EMOJI_MAP.get("雷").unwrap(), ":zap:");
    }

    #[test]
    fn test_weather_emoji_map() {
        assert_eq!(WEATHER_EMOJI_MAP.get("01").unwrap(), ":sunny:");
        assert_eq!(WEATHER_EMOJI_MAP.get("02").unwrap(), ":mostly_sunny:");
    }
}
