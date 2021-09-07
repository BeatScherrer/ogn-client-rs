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
    .build(Root::builder().appender("stdout").build(LevelFilter::Debug))
    .unwrap();

  log4rs::init_config(config).unwrap();

  // ------------------------------------------------------------------------------
  // Here comes the interesting part
  // ------------------------------------------------------------------------------
  let callback = |message: &str| {
    println!("callback: {}", message);
  };

  // pass the callback to the client!
  let mut client = APRSClient::new("aprs.glidernet.org", PORT::FULLFEED, Box::new(callback));

  // log into the network
  let login_data = LoginData::new().user_name("Beat");
  client.login(login_data)?;

  // example of sending a position message
  // client.send_message("user AE5PL-TS pass -1 vers testsoftware 1.0_05 filter r/33.25/-96.5/50").unwrap();

  // example of sending a status message
  // TODO

  // TODO think about where to dispatch the reading
  client.run();
  Ok(())
}
