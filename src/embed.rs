use std::collections::HashMap;

use lazy_static::lazy_static;
use rust_embed::RustEmbed;
use std::str::from_utf8;

const EMOJI_MAP_TOML: &str = "emoji_map.toml";

#[derive(RustEmbed)]
#[folder = "embed/"]
struct Embed;

lazy_static! {
    pub static ref EMOJI_MAP: HashMap<String, String> = init_emoji_map();
}

fn init_emoji_map() -> HashMap<String, String> {
    let file = Embed::get(EMOJI_MAP_TOML).expect("emoji_map not found.");
    let raw = from_utf8(file.as_ref()).expect("emoji_map couldn't open.");
    toml::from_str(raw).expect("emoji_map couldn't parse.")
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_emoji_map() {
        assert_eq!(EMOJI_MAP.get("01").unwrap(), ":sunny:");
        assert_eq!(EMOJI_MAP.get("02").unwrap(), ":mostly_sunny:");
    }
}
