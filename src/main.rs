mod parser;
use parser::Parse;
use std::io::Error;
use log::debug;

use ogn_client_rs::{APRSClient, LoginData, PORT};

fn main() -> Result<(), Error> {
  log4rs::init_file("logger_config.yaml", Default::default()).unwrap();

  // ------------------------------------------------------------------------------
  // Here comes the interesting part
  // ------------------------------------------------------------------------------
  let callback = |message: &str| {
    // This does not work completely yet...
    let result = parser::OgnTransmission::parse(message);
    if let Some(v) = result {
      debug!("read message");
      println!("{:#?}", v);
    }
  };

  // pass the callback to the client!
  let client = APRSClient::new("aprs.glidernet.org", PORT::FILTER, Box::new(callback));

  // log into the network
  let login_data = LoginData::new().user_name("Beat").pass_code("28915");

  {
    let mut locked = client.lock().unwrap();

    while !locked.is_connected() {
      let _ = locked.connect();

      std::thread::sleep(std::time::Duration::from_secs(1));
    }

    locked.login(login_data)?;
  }

  APRSClient::run(client.clone());

  // set filter
  client.lock().unwrap().set_filter("r/47/7/100").unwrap();

  println!("keeping client alive");
  loop {
    std::thread::sleep(std::time::Duration::from_secs(1));
  }
}
