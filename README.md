## rusty-slack-weather-status

Set a weather on a Slack users's status.

![image](https://user-images.githubusercontent.com/12206768/107185370-ae90ee00-6a25-11eb-98d5-39a10e198073.png)

```
cargo run -- -u <tenki.jp URL: e.g. https://tenki.jp/forecast/3/16/4410/13120> -t <Slack Web API token e.g. xoxp-...>
```

or

```
cargo build --release

crontab -e
*/10 * * * * /<repository_path>/git/rusty-slack-weather-status/target/release/rusty-slack-weather-status -u https://tenki.jp/forecast/3/16/4410/13120 -t <Slack Web API token e.g. xoxp-...> >/dev/null 2>&1
```

## Dry Run

```sh
RUST_LOG=info cargo run -- -u https://tenki.jp/forecast/3/16/4410/13120 -t <Slack Web API token> --dry

[2021-07-30T09:40:59Z INFO  rusty_slack_weather_status] Forecast { place: "練馬区", date_time: "30日16:00", advisory: Some("洪水"), warning: None, emergency_warning: None, weather: "雨", weather_icon_stem: "15_n", high_temp: 29, high_temp_diff: TempDiff { temp_diff: -4 }, low_temp: 26, low_temp_diff: TempDiff { temp_diff: 1 } }
[2021-07-30T09:40:59Z INFO  rusty_slack_weather_status] ":warning:", "練馬区: 洪水注意報 :umbrella:: 雨 最高: 29℃[-4] 最低: 26℃[+1] 発表: 30日16:00"
```

## Get Slack Web API token

1. https://api.slack.com/apps > Create New App

1. Permissions > User Token Scopes > `users.profile:write`

1. See https://api.slack.com/apps/A029YB02KEG/oauth? > OAuth Tokens
