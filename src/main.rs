extern crate clap;
#[macro_use]
extern crate log;

use clap::{App, Arg};
use rusty_slack_weather_status::get_forecast;

fn main() {
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
                .takes_value(true),
        )
        .get_matches();

    let tenkijp_url = matches.value_of("URL").unwrap();
    info!("tenki.jp URL: {}", tenkijp_url);
    let forecast = get_forecast(tenkijp_url);
    println!("{:?}", forecast);
}
