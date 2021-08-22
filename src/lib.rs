/*
#ogn-client-rs
TODO:
- restrict buffer to 512Bytes
- add passcode generationo
- Check for login status before reading data

##aprs-is notes:
- constant information should only be sent every 5 minutes
[ogn-wiki](http://wiki.glidernet.org/aprs-interaction-examples)
- After every 20s a heartbeat is sent from the server, try to reconnect
after 1min of not receiving the heartbeat
*/

use log::{debug, info, warn};
use std::io::{BufRead, BufReader, LineWriter, Write};
use std::net::TcpStream;
use chrono::prelude::*;

use geocoding::Coordinate;

// parses the received string to an appropriate obejct
pub trait Parse {
  type Item;

  fn parse(string: &str) -> Self::Item;
}

#[derive(Debug, PartialEq)]
struct OgnMetaData {
  m_pilot_name: Option<String>,
  m_manufacturer: Option<String>,
  m_model: Option<String>,
  m_type: Option<String>,
  m_serial_number: Option<u32>,
  m_competition_id: Option<String>,
  m_competition_class: Option<String>,
  m_competition_task: Option<String>,
  m_base_airfield: Option<String>,
  m_in_case_of_emergency: Option<String>,
  m_pilot_id: Option<String>,
  m_hardware: Option<String>,
  m_software: Option<String>,
}

impl OgnMetaData {
  pub fn new() -> Self {
    OgnMetaData {
      m_pilot_name: None,
      m_manufacturer: None,
      m_model: None,
      m_type: None,
      m_serial_number: None,
      m_competition_id: None,
      m_competition_class: None,
      m_competition_task: None,
      m_base_airfield: None,
      m_in_case_of_emergency: None,
      m_pilot_id: None,
      m_hardware: None,
      m_software: None,
    }
  }
}

// describes an ogn position
#[derive(Debug, PartialEq)]
struct OgnPosition {
  meta: OgnMetaData,
  timestamp: DateTime<Utc>,
  position: Coordinate<f32>,
}

impl OgnPosition {
  pub fn new(timestamp: DateTime<Utc>, position: Coordinate<f32>) -> Self {
    OgnPosition{
      meta: OgnMetaData::new(),
      timestamp: timestamp,
      position: position
    }
  }
}


impl Parse for OgnPosition {
  type Item = Self;

  fn parse(message: &str) -> Self {
    OgnPosition::new(Utc::now(), Coordinate{x: 45.0, y: 13.0})
  }
}

#[repr(u16)]
pub enum PORT {
  FULLFEED = 10152,
  FILTER = 14580,
}


pub struct APRSClient {
  m_reader: BufReader<TcpStream>,
  m_writer: LineWriter<TcpStream>,
  // m_parser: Box<dyn Parse> // TODO fix this dynamic type's associated item?
}

impl APRSClient {
  pub fn new(target: &str, port: PORT) -> Self {
    // ip addr
    let port = port as u16;
    info!("creating aprs client with target '{}:{}'", target, port);

    let connection = TcpStream::connect((target, port)).unwrap();

    APRSClient {
      m_writer: LineWriter::new(connection.try_clone().unwrap()),
      m_reader: BufReader::new(connection), // m_buffer: &mut[0; 128],
    }
  }

  pub fn login(&mut self, login_data: LoginData) -> Result<(), std::io::Error> {
    info!("login with following data:\n{:#?}", &login_data);
    let login_message = APRSClient::create_aprs_login(login_data);

    self.send_message(login_message.as_str())?;

    // read

    Ok(())
  }

  pub fn login_default(&mut self) -> Result<(), std::io::Error> {
    let login_data = LoginData::new(None, None, None, None);
    self.login(login_data)?;

    Ok(())
  }

  pub fn run(&mut self) {
    info!("starting the client...");
    println!("a {}", self.read().unwrap());

    self
      .send_message("user BEAT pass -1 vers testsoftware 1.0_05 filter r/33.25/-96.5/50\r\n")
      .unwrap();
    println!("{}", self.read().unwrap());

    loop {
      println!("{}", self.read().unwrap());
      // if !self.m_logged_in {
      //   warn!("not logged in! make sure to be logged in");
      // } else {
      //   // heartbeat
      //   self.send_heart_beat();
      //   let result = self.read().unwrap();
      //   info!("{:?}", result);
      // }
    }
  }

  /// Send bytes and return the answer
  pub fn send_message(&mut self, message: &str) -> Result<(), std::io::Error> {
    let mut full_message = String::new();
    debug!("sending message: '{}'", message);

    full_message.push_str(message);
    full_message.push_str("\r\n");
    self.m_writer.write_all(full_message.as_bytes())?;
    self.m_writer.flush()?;

    Ok(())
  }
  pub fn send_heart_beat(&mut self) {
    self.send_message("#keepalive").unwrap();
  }

  fn read(&mut self) -> Result<String, std::io::Error> {
    debug!("reading message ...");
    let mut string_buffer = String::new();

    self.m_reader.read_line(&mut string_buffer)?;
    string_buffer = string_buffer.trim_end().to_string();

    Ok(string_buffer)
  }

  fn create_aprs_login(login_data: LoginData) -> String {
    format!(
      "user {} pass {} vers {} {} filter r/33.25/-96.5/50",
      login_data.user_name, login_data.pass_code, login_data.app_name, login_data.app_version
    )
  }
}

impl Drop for APRSClient {
  fn drop(&mut self) {
    info!("...terminating the aprs client!");
  }
}

#[derive(Debug)]
pub struct LoginData<'a> {
  pub user_name: &'a str,
  pub pass_code: &'a str,
  pub app_name: &'a str,
  pub app_version: &'a str,
}

impl<'a> LoginData<'a> {
  pub fn new(
    mut user_name: Option<&'a str>,
    mut pass_code: Option<&'a str>,
    mut app_name: Option<&'a str>,
    mut app_version: Option<&'a str>,
  ) -> LoginData<'a> {
    Self {
      user_name: user_name.get_or_insert("BEAT"),
      pass_code: pass_code.get_or_insert("-1"),
      app_name: app_name.get_or_insert(env!("CARGO_PKG_NAME")),
      app_version: app_version.get_or_insert(env!("CARGO_PKG_VERSION")),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parser_test() {
    // test message from ogn wiki, make sure this corresponds to the actual specifications
    let test_message = r#"OGN82149C>OGNTRK,qAS,OxfBarton:/130208h5145.95N/00111.50W'232/000/A=000295 !W33! id3782149C +000fpm -4.3rot FL000.00 55.0dB 0e -3.7kHz gps3x5"#;
    // create the expected output
    let expected = OgnPosition{
      meta: OgnMetaData::new(),
      timestamp: Utc::now(),
      position: Coordinate{x: 0.0, y: 0.0}
    };

    let parsed_position = OgnPosition::parse(test_message);

    assert_eq!(expected, parsed_position);
  }
}
