use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;

use std::io::Error;

use ogn_client_rs::{APRSClient, LoginData, PORT};

fn main() -> Result<(), Error> {
  //configure loggers
  let stdout = ConsoleAppender::builder()
    .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
    .build();

  let config = Config::builder()
    .appender(Appender::builder().build("stdout", Box::new(stdout)))
    .logger(Logger::builder().build("ogn_client_rs", LevelFilter::Debug))
    .build(Root::builder().appender("stdout").build(LevelFilter::Warn))
    .unwrap();

  log4rs::init_config(config).unwrap();

  let mut client = APRSClient::new("aprs.glidernet.org", PORT::FULLFEED);

  LoginData::new(None, None, None, None);

  // client.login_default()?;
  // client.send_message("user AE5PL-TS pass -1 vers testsoftware 1.0_05 filter r/33.25/-96.5/50").unwrap();

  client.run();

  Ok(())
}
