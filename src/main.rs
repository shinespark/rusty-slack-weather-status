extern crate clap;
#[macro_use]
extern crate log;

use clap::{App, Arg};
use rusty_slack_weather_status::forecast::TenkiJpForecast;
use rusty_slack_weather_status::slack::SlackRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let matches = App::new("Rusty Slack Weather Status")
        .version("0.9")
        .about("Set the weather on your Slack status")
        .arg(
            Arg::with_name("URL")
                .short("u")
                .long("url")
                .help("Sets a tenki.jp url. e.g.) https://tenki.jp/forecast/3/16/4410/13113")
                .required(true)
                .takes_value(true))
        .arg(
            Arg::with_name("SLACK_TOKEN")
                .short("t")
                .long("token")
                .help(
                    "Sets a slack token. e.g.) xoxp-***********-************-************-********************************")
                .required(true)
                .takes_value(true)
        )
        .get_matches();

    let tenki_jp_url = matches.value_of("URL").unwrap();
    info!("tenki.jp URL: {}", tenki_jp_url);

    let tenki_jp_forecast = TenkiJpForecast::get(tenki_jp_url).await?;
    let forecast = tenki_jp_forecast.parse()?;
    info!("{:?}", forecast);

    let token = matches.value_of("SLACK_TOKEN").unwrap();
    let slack_request = SlackRequest::new(token);
    let (slack_result, res_body) = slack_request
        .update_status(":sunny:", &forecast.weather)
        .await?;
    info!("{:?}", slack_result);
    info!("{:?}", res_body);
    Ok(())
}
