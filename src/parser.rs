use chrono::prelude::*;
use geocoding::Coordinate;
use log::{debug, error, info};
use regex::Regex;
use std::fmt::Debug;

pub trait Parse {
  type Item: Debug;

  fn parse(string: &str) -> Option<Self::Item>;
}

#[derive(Debug, PartialEq)]
pub struct OgnTransmission {
  pub header: OgnHeader,
  pub message: OgnMessage,
}

#[derive(Debug, PartialEq)]
pub struct OgnHeader {
  sender_id: String,
  receiver: String,
  transmission_method: String,
}

#[derive(Debug, PartialEq)]
pub struct OgnMessage {
  timestamp: DateTime<Utc>,
  position: Coordinate<f32>,
  ground_speed: f32,
  ground_turning_rate: f32,
  climb_rate: f32,
  altitude: f32,
  ground_track: u16,
  gps_accuracy: String,
  id: String,
}

impl Parse for OgnTransmission {
  type Item = Self;

  fn parse(message: &str) -> Option<Self> {
    if message.starts_with("#") {
      return None;
    }

    // first: split at ':' to split the header from the message
    let splits: Vec<&str> = message.split(':').collect();

    let header = parse_header(splits[0]);
    let message = parse_message(splits[1]);

    Some(OgnTransmission {
      header: header,
      message: message,
    })
  }
}

fn parse_header(header: &str) -> OgnHeader {
  let header_splits: Vec<&str> = header.split('>').collect();
  let sender_id = header_splits[0];

  let header_splits: Vec<&str> = header_splits[1].split(',').collect();

  let receiver = header_splits[0];
  let transmission_method = header_splits[1];

  OgnHeader {
    sender_id: sender_id.to_string(),
    receiver: receiver.to_string(),
    transmission_method: transmission_method.to_string(),
  }
}

fn parse_message(message: &str) -> OgnMessage {
  debug!("{:#?}", message);

  // first split the message at the extra field separator
  let regex = Regex::new("!W[0-9]+!").expect("bad regex");
  let splits: Vec<&str> = regex.split(&message).collect();

  for &split in &splits {
    println!("## {}", &split);
  }

  /*
  TODO support variable length parsing of extra fields:
  The fields are not always present. Therefore a dynamic splits must be applied.
    - Split second split again at ' '
    - Add field description, is the order always given or must the field type
      be detected dynamically??
    - Use regex to match the types:
      - Add map with field types and regex patterns
  */

  let aprs_part = splits[0];
  let ogn_extra_part = splits[1];

  let time_string = &aprs_part[1..7];

  let lat = &aprs_part[8..15].replace(".", "");
  let lon = &aprs_part[17..25].replace(".", "");

  let ground_track = &aprs_part[27..30];
  let ground_speed = &aprs_part[31..34];

  let id = &ogn_extra_part[1..11];
  let climb_rate_string = &ogn_extra_part[12..20];
  let ground_turning_rate = &ogn_extra_part[20..24];
  let altitude_string = &ogn_extra_part[28..36];

  // What are these fields supposed to contain?
  // let signal_strength = &ogn_extra_part[37..43];
  // let what_is_this = &ogn_extra_part[44..46];
  // let what_frequency_difference_is_this = &ogn_extra_part[47..54];

  let gps_accuracy = &ogn_extra_part[55..];

  // assemble the message
  OgnMessage {
    timestamp: Utc.ymd(2021, 8, 22).and_hms(
      time_string[0..2].parse().unwrap(),
      time_string[2..4].parse().unwrap(),
      time_string[5..].parse().unwrap(),
    ),
    position: Coordinate {
      x: lat.parse::<f32>().unwrap() / 10000.0,
      y: lon.parse::<f32>().unwrap() / 10000.0,
    },
    ground_speed: ground_speed.parse().unwrap(),
    ground_turning_rate: ground_turning_rate.parse().unwrap(),
    climb_rate: climb_rate_string[0..4].parse().unwrap(),
    altitude: altitude_string[2..].parse().unwrap(),
    ground_track: ground_track.parse().unwrap(),
    gps_accuracy: gps_accuracy.to_string(),
    id: id.to_string(),
  }
}

pub fn parse_login_answer(login_answer: &str) -> bool {
  let re = Regex::new(r"^# logresp (\w+) (verified)").unwrap();

  match re.find(login_answer) {
    Some(_) => true,
    _ => false,
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
      timestamp: Utc.ymd(2021, 08, 22).and_hms(13, 02, 08),
      position: Coordinate {
        x: 51.4595,
        y: 001.1150,
      },
      altitude: 0.0,
      climb_rate: 0.0,
      ground_speed: 0.0,
      ground_turning_rate: -4.3,
      ground_track: 232,
      gps_accuracy: "gps3x5".to_string(),
      id: "id3782149C".to_string(),
    };

    let expected = OgnTransmission {
      header: header,
      message: message,
    };

    let parsed_position = OgnTransmission::parse(test_message).unwrap();

    assert_eq!(expected, parsed_position);
  }

  #[test]
  fn parser_login_answer() {
    assert!(parse_login_answer(r"# logresp Beat verified, server GLIDERN2") == true);

    assert!(parse_login_answer(r"# logresp Beat unverified, server GLIDERN1") == false);
    assert!(parse_login_answer("random string") == false);
  }
}
