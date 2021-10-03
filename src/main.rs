use std::io::Error;

use ogn_client_rs::{APRSClient, LoginData, PORT};

mod parser;

fn main() -> Result<(), Error> {
  log4rs::init_file("logger_config.yaml", Default::default()).unwrap();

  // ------------------------------------------------------------------------------
  // Here comes the interesting part
  // ------------------------------------------------------------------------------
  let callback = |message: &str| {
    println!("callback: {}", message);

    // This does not work yet...
    // println! {"{:#?}", OgnTransmission::parse(message).unwrap()};
  };

  // pass the callback to the client!
  let client = APRSClient::new("aprs.glidernet.org", PORT::FULLFEED, Box::new(callback));

  // log into the network
  let login_data = LoginData::new().user_name("Beat").pass_code("28915");
  client.lock().unwrap().login(login_data)?;
  APRSClient::run(client.clone());

  // example of sending a position message
  // client
  //   .send_message("user AE5PL-TS pass -1 vers testsoftware 1.0_05 filter r/33.25/-96.5/50")
  //   .unwrap();

  // example of sending a status message
  // TODO

  println!("keeping client alive");
  std::thread::sleep(std::time::Duration::from_secs(5));

  Ok(())
}
