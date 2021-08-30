use chrono::prelude::*;
use geocoding::Coordinate;
use std::str::FromStr;

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

// describes an ogn position
#[derive(Debug, PartialEq)]
struct OgnMessage {
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

  fn parse(message: &str) -> Self {
    // first: split at ':' to split the header from the message
    let splits: Vec<&str> = message.split(':').collect();

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

  let header_splits: Vec<&str> = header_splits[1].split(',').collect();

  let receiver = header_splits[0];
  let transmission_method = header_splits[1];

  OgnHeader {
    sender_id: sender_id.to_string(),
    receiver: receiver.to_string(),
    transmission_method: transmission_method.to_string(),
  }
}

// TODO proper error handling
fn parse_message(message: &str) -> OgnMessage {
  // first split the message at the extra field separator
  let splits: Vec<&str> = message.split("!W33!").collect();

  let aprs_part = splits[0];
  let ogn_extra_part = splits[1];

  let time_string = &aprs_part[1..7];

  let lat = &aprs_part[8..15].replace(".", "");
  let lon = &aprs_part[17..25].replace(".", "");

  let ground_track = &aprs_part[27..30];
  let ground_speed = &aprs_part[31..34];

  // println!("#####################  {:#?}", lat);

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

    let parsed_position = OgnTransmission::parse(test_message);

    assert_eq!(expected, parsed_position);
  }
}
