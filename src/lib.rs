/*
#ogn-client-rs
TODO:
- restrict buffer to 512Bytes
- add passcode generation
- Check for login status before reading data
- Move the Ogn Structures to seperate module
- Make sure the lat/long conversion to coordinate is correct
- finish parsing of the header
- finish parsing of the message
- add send status
- add filter functionality
- replace String with &str where possible

##aprs-is notes:
- constant information should only be sent every 5 minutes
[ogn-wiki](http://wiki.glidernet.org/aprs-interaction-examples)
- After every 20s a heartbeat is sent from the server, try to reconnect
after 1min of not receiving the heartbeat

an example of a ogn message looks as follows:
```
OGN82149C>OGNTRK,qAS,OxfBarton:/130208h5145.95N/00111.50W'232/000/A=000295 !W33! id3782149C +000fpm -4.3rot FL000.00 55.0dB 0e -3.7kHz gps3x5
```

where the header `OGN82149C>OGNTRK,qAS,OxfBarton` and message `/130208h5145.95N/00111.50W'232/000/A=000295 !W33! id3782149C +000fpm -4.3rot FL000.00 55.0dB 0e -3.7kHz gps3x5`
are seperated with a `:`. The fields until `!W33!` are pure APRS format and after are "comments" which carry ogn specific extra information

therefore the default parser parses the message into a `OgnTransmission` struct which consists of `OgnHeader` and `OgnMessage`.

The parser can be overriden but since it is a specified format the default parsers should do just fine. Conversions to
other data structures can be don after parsing. When you want to parse directly into your data structure just implement
the `Parse` trait which requires a `fn parse(transmission: &str) -> YourType`.

*/

use chrono::prelude::*;
use log::{debug, info};
use std::io::{BufRead, BufReader, LineWriter, Write};
use std::net::TcpStream;
use std::str::FromStr;

use geocoding::Coordinate;

// parses the received string to an appropriate obejct
pub trait Parse {
  type Item;

  fn parse(string: &str) -> Self::Item;
}

#[derive(Debug, PartialEq)]
struct OgnTransmission {
  header: OgnHeader,
  message: OgnMessage,
}

#[derive(Debug, PartialEq)]
struct OgnStatusMessage {
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

impl OgnStatusMessage {
  pub fn new() -> Self {
    OgnStatusMessage {
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
struct OgnMessage {
  timestamp: DateTime<Utc>,
  position: Coordinate<f32>,
  ground_speed: f32,
  ground_turning_rate: f32,
  climb_rate: f32,
  altitude: u32,
  ground_track: u16,
  gps_accuracy: String,
}

// impl OgnMessage {
//   pub fn new(timestamp: NaiveTime, position: Coordinate<f32>) -> Self {
//     OgnMessage {
//       timestamp: timestamp,
//       position: position,
//     }
//   }
// }

impl Parse for OgnTransmission {
  type Item = Self;

  fn parse(message: &str) -> Self {
    // first: split at ':' to split the header from the message
    let mut splits: Vec<&str> = message.split(':').collect();

    let header = parse_header(splits[0]);
    let message = parse_message(splits[1]);

    OgnTransmission {
      header: header,
      message: message,
    }
  }
}

fn parse_header(header: &str) -> OgnHeader {
  let header_splits: Vec<&str> = header.split('>').collect();
  let sender_id = header_splits[0];

  let header_splits: Vec<&str> = header_splits[0].split(',').collect();
  let receiver = header_splits[0];
  let transmission_method = header_splits[1];
  let signal_strength = header_splits[2];

  OgnHeader {
    sender_id: sender_id.to_string(),
    receiver: receiver.to_string(),
    transmission_method: transmission_method.to_string(),
  }
}

fn parse_message(message: &str) -> OgnMessage {
  // first split the message at the extra field separator
  let splits: Vec<&str> = message.split("!W66").collect();

  let aprs_part = splits[0];
  let ogn_extra_part = splits[1];

  // split at / to get the main aprs fields
  let aprs_splits: Vec<&str> = aprs_part.split('/').collect();

  //
  let extra_ogn_splits: Vec<&str> = ogn_extra_part.split('!').collect();

  for field in extra_ogn_splits {
    println!("{:#?}", field);
  }

  OgnMessage {
    timestamp: DateTime::from_str("10:10:10").unwrap(),
    position: Coordinate { x: 0.0, y: 1.1 },
    ground_speed: 250.0,
    ground_turning_rate: 1.0,
    climb_rate: 2.5,
    altitude: 1500,
    ground_track: 160,
    gps_accuracy: "gps4x5".to_string(),
  }
}

#[derive(Debug, PartialEq)]
struct OgnHeader {
  sender_id: String,
  receiver: String,
  transmission_method: String,
}

enum OgnStatusField {
  PilotName,
  Manuf,
  Model,
  Type,
  SerialNumber,
  Registration,
  CompetitionId,
  CompetitionClass,
  CompetitionTask,
  BaseAirfield,
  InCaseOfEmergency,
  PilotId,
  Hardware,
  Software,
}

impl FromStr for OgnStatusField {
  type Err = ();

  fn from_str(input: &str) -> Result<OgnStatusField, Self::Err> {
    match input {
      "Pilot" => Ok(OgnStatusField::PilotName),
      "Manuf" => Ok(OgnStatusField::Manuf),
      "Model" => Ok(OgnStatusField::Model),
      "Type" => Ok(OgnStatusField::Type),
      "SN" => Ok(OgnStatusField::SerialNumber),
      "Reg" => Ok(OgnStatusField::Registration),
      "ID" => Ok(OgnStatusField::CompetitionId),
      "Class" => Ok(OgnStatusField::CompetitionClass),
      "Task" => Ok(OgnStatusField::CompetitionTask),
      "Base" => Ok(OgnStatusField::BaseAirfield),
      "ICE" => Ok(OgnStatusField::InCaseOfEmergency),
      "PilotID" => Ok(OgnStatusField::PilotId),
      "Hard" => Ok(OgnStatusField::Hardware),
      "Soft" => Ok(OgnStatusField::Software),
      _ => Err(()),
    }
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
    let header = OgnHeader {
      sender_id: "OGN82149C".to_string(),
      receiver: "OGNTRK".to_string(),
      transmission_method: "qAS".to_string(),
    };

    let message = OgnMessage {
      timestamp: DateTime::from_str("13:02:08").unwrap(),
      position: Coordinate {
        x: 54.4595,
        y: 001.1150,
      },
      altitude: 0,
      climb_rate: 0.0,
      ground_speed: 0.0,
      ground_turning_rate: -4.3,
      ground_track: 232,
      gps_accuracy: "gps3x5".to_string(),
    };

    let expected = OgnTransmission {
      header: header,
      message: message,
    };

    let parsed_position = OgnTransmission::parse(test_message);

    assert_eq!(expected, parsed_position);
  }
}
