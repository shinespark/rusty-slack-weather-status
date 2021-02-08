## rusty-slack-weather-status

Set a weather on a Slack users's status.

![image](https://user-images.githubusercontent.com/12206768/107185370-ae90ee00-6a25-11eb-98d5-39a10e198073.png)

```
cargo run -- -u <tenki.jp URL: e.g. https://tenki.jp/forecast/3/16/4410/13120> -t <Slack Web API token e.g. xoxp-...>
```

## Get Slack Web API token

1. https://api.slack.com/apps > Create New App

1. Permissions > User Token Scopes > `users.profile:write`
