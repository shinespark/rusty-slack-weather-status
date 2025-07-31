extern crate clap;
#[macro_use]
extern crate log;

use clap::{App, Arg};
use rusty_slack_weather_status::models::tenki_jp_forecast::TenkiJpForecast;
use rusty_slack_weather_status::slack::SlackRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let matches = App::new("Rusty Slack Weather Status")
        .version("0.99")
        .about("Set the weather on your Slack status")
        .arg(
            Arg::with_name("URL")
                .short("u")
                .long("url")
                .help("Sets a tenki.jp url. e.g.) https://tenki.jp/forecast/3/16/4410/13113")
                .required(true)
                .takes_value(true))
        .arg(
            Arg::with_name("DRY RUN")
                .short("d")
                .long("dry")
                .help("Don't send to the Slack.")
        )
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
    let tenki_jp_forecast = TenkiJpForecast::get(tenki_jp_url).await?;
    let forecast = tenki_jp_forecast.parse()?;

    let is_dry_run = matches.is_present("DRY RUN");
    match is_dry_run {
        true => {
            println!("{forecast:?}");
            println!(
                "{:?}, {:?}",
                &forecast.build_emoji(),
                &forecast.build_text()
            )
        }
        false => {
            let token = matches.value_of("SLACK_TOKEN").unwrap();
            let slack_request = SlackRequest::new(token);
            let (_status_code, res) = slack_request
                .update_status(&forecast.build_emoji(), &forecast.build_text())
                .await?;
            info!("{:?}", res);
        }
    }

    Ok(())
}
