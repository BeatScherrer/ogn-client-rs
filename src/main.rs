use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;

use std::io::Error;

use ogn_client_rs::{APRSClient, LoginData};

fn main() -> Result<(), Error> {
  //configure loggers
  let stdout = ConsoleAppender::builder()
    .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
    .build();

  let config = Config::builder()
    .appender(Appender::builder().build("stdout", Box::new(stdout)))
    .logger(Logger::builder().build("ogn_client_rs", LevelFilter::Debug))
    .build(Root::builder().appender("stdout").build(LevelFilter::Debug))
    .unwrap();

  log4rs::init_config(config).unwrap();

  let mut client = APRSClient::new("aprs.glidernet.org", 14580);

  // let login_message = String::from("user NOCALL pass -1");
  // let response = client.send_message(&login_message)?;
  // println!("{:?}", response);
  let login_data = LoginData{
    user_name: "beat",
    pass_code: "", // TODO
    app_name: "aprs-client-rs",
    app_version: env!("CARGO_PKG_VERSION")
  };

  client.login()

  client.run();

  Ok(())
}
