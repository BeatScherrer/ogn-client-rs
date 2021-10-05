use chrono::prelude::*;
use geocoding::Coordinate;
use log::{debug, error};
use regex::Regex;
use std::fmt::Debug;

pub trait Parse {
  type Item: Debug;

  fn parse(string: &str) -> Option<Self::Item>;
}

#[derive(Debug, PartialEq)]
pub struct OgnTransmission {
  pub header: OgnHeader,
  pub body: OgnBody,
}

#[derive(Debug, PartialEq)]
pub struct OgnHeader {
  sender_id: String,
  receiver: String,
  transmission_method: String,
}

#[derive(Debug, PartialEq)]
pub struct OgnBody {
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
      debug!("received comment: {}", message);
      return None;
    }

    // TODO add more checks before actually parsing

    // first: split at ':' to split the header from the message
    let splits: Vec<&str> = message.split(':').collect();

    let header = parse_header(splits[0]);
    if let None = header {
      error!("error while parsing header, skipping message");
      return None;
    }

    let header = header.unwrap();

    let body = parse_body(splits[1]);
    if let None = body {
      error!("error while parsing body, skipping message");
      return None;
    }

    let body = body.unwrap();

    Some(OgnTransmission {
      header: header,
      body: body,
    })
  }
}

fn parse_header(header: &str) -> Option<OgnHeader> {
  // parse header with regex
  let regex = Regex::new(r"^(?P<id>\w+)>(?P<target>.+),(?P<protocol>\w+),(?P<receiver>\w+)$")
    .expect("error in header regex");

  let captures = regex.captures(header)?;

  let sender_id = captures.name("id").unwrap().as_str();
  let _target = captures.name("target").unwrap().as_str();
  let transmission_method = captures.name("protocol").unwrap().as_str();
  let receiver = captures.name("receiver").unwrap().as_str();

  Some(OgnHeader {
    sender_id: sender_id.to_string(),
    receiver: receiver.to_string(),
    transmission_method: transmission_method.to_string(),
  })
}

fn parse_body(body: &str) -> Option<OgnBody> {
  // first split the message at the extra field separator
  // note: the part before !W..! is a regular aprs expression
  // while afterwards is an ogn extension
  let regex = Regex::new("!W[0-9]+!").expect("bad splitting regex");
  let splits: Vec<&str> = regex.split(&body).collect();

  for &split in &splits {
    println!("## {}", &split);
  }

  let aprs_part = splits[0].trim();
  let ogn_extra_part = splits[1].trim();

  // parse aprs part
  let aprs_regex = Regex::new(
    r"^/(?P<time>\d{6})h(?P<lat>.*)[NS]/(?P<lon>.*)[EW]['^](?P<ground_track>\d{3})/(?P<ground_speed>\d{3})/A=(?P<altitude>\d{6})$",
  )
  .expect("bad aprs regex");

  let captures = aprs_regex.captures(aprs_part);
  if let None = captures {
    println!("error while parsing body! {}", aprs_part);
    return None;
  }

  let captures = captures.unwrap();

  // timestamp
  let time_string = captures.name("time").unwrap().as_str();
  let timestamp = Utc::today().and_hms(
    time_string[..2].parse().unwrap(),
    time_string[2..4].parse().unwrap(),
    time_string[4..].parse().unwrap(),
  );

  // lat
  let lat = captures.name("lat").unwrap().as_str();
  let lat = lat.replace(".", "");

  // lon
  let lon = captures.name("lon").unwrap().as_str();
  let lon = lon.replace(".", "");

  // ground track
  let ground_track = captures.name("ground_track").unwrap().as_str();

  // ground speed
  let ground_speed = captures.name("ground_speed").unwrap().as_str();

  // altitude
  let altitude: f32 = captures.name("altitude").unwrap().as_str().parse().unwrap();

  // ------------------------------------------------------------------------------
  // parse ogn part
  // ------------------------------------------------------------------------------
  let ogn_parser = Regex::new(
    r"^id(?P<id>\w{8}) (?P<climb>[+-].*)fpm (?P<rot>[+-].*)rot (.*)dB (.*)kHz gps(?P<accuracy>\dx\d)",
  )
  .expect("bad ogn message regex");

  let captures = ogn_parser.captures(ogn_extra_part);
  if let None = captures {
    println!("error while parsing body: {}", ogn_extra_part);
    return None;
  }

  let captures = captures.unwrap();

  let id = captures.name("id").unwrap().as_str();
  let climb_rate: f32 = captures.name("climb").unwrap().as_str().parse().unwrap();
  let rotation_rate: f32 = captures.name("rot").unwrap().as_str().parse().unwrap();
  let gps_accuracy = captures.name("accuracy").unwrap().as_str();

  // assemble the message
  Some(OgnBody {
    timestamp: timestamp,
    position: Coordinate {
      x: lat.parse::<f32>().unwrap() / 10000.0,
      y: lon.parse::<f32>().unwrap() / 10000.0,
    },
    ground_speed: ground_speed.parse().unwrap(),
    ground_turning_rate: rotation_rate,
    climb_rate: climb_rate,
    altitude: altitude,
    ground_track: ground_track.parse().unwrap(),
    gps_accuracy: String::from(gps_accuracy),
    id: id.to_string(),
  })
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
  fn parse_full_message() {
    // test message from ogn wiki, make sure this corresponds to the actual specifications
    let test_message = r#"OGN82149C>OGNTRK,qAS,OxfBarton:/130208h5145.95N/00111.50W'232/000/A=000295 !W33! id3782149C +000fpm -4.3rot FL000.00 55.0dB 0e -3.7kHz gps3x5"#;

    // create the expected output
    let header = OgnHeader {
      sender_id: "OGN82149C".to_string(),
      receiver: "OxfBarton".to_string(),
      transmission_method: "qAS".to_string(),
    };

    let body = OgnBody {
      timestamp: Utc::today().and_hms(13, 02, 08),
      position: Coordinate {
        x: 51.4595,
        y: 001.1150,
      },
      altitude: 295.0,
      climb_rate: 0.0,
      ground_speed: 0.0,
      ground_turning_rate: -4.3,
      ground_track: 232,
      gps_accuracy: "3x5".to_string(),
      id: "3782149C".to_string(),
    };

    let expected = OgnTransmission {
      header: header,
      body: body,
    };

    let parsed_position = OgnTransmission::parse(test_message).unwrap();

    assert_eq!(expected, parsed_position);
  }

  #[test]
  fn parse_login_answer_test() {
    assert!(parse_login_answer(r"# logresp user verified, server GLIDERN2") == true);

    assert!(parse_login_answer(r"# logresp user unverified, server GLIDERN1") == false);
    assert!(parse_login_answer("random string") == false);
  }

  #[test]
  fn parse_message_header_test() {
    let expected_header1 = OgnHeader {
      sender_id: "LFNW".to_string(),
      receiver: "GLIDERN5".to_string(),
      transmission_method: "qAC".to_string(),
    };

    let ogn_message_header1 = r"LFNW>APRS,TCPIP*,qAC,GLIDERN5";
    let parsed_header1 = parse_header(ogn_message_header1);
    assert_eq!(parsed_header1.unwrap(), expected_header1);

    let expected_header2 = OgnHeader {
      sender_id: "LFNW".to_string(),
      receiver: "GLIDERN5".to_string(),
      transmission_method: "qAC".to_string(),
    };
    let ogn_message_header2 = r"LFNW>APRS,TCPIP*,qAC,GLIDERN5";
    let parsed_header2 = parse_header(ogn_message_header2);
    assert_eq!(parsed_header2.unwrap(), expected_header2);

    let expected_header3 = OgnHeader {
      sender_id: "FLRDDE626".to_string(),
      receiver: "EGHL".to_string(),
      transmission_method: "qAS".to_string(),
    };
    let ogn_message_header3 = r"FLRDDE626>APRS,qAS,EGHL";
    let parsed_header3 = parse_header(ogn_message_header3);
    assert_eq!(parsed_header3.unwrap(), expected_header3);
  }

  #[test]
  fn parse_message_body_test() {
    let ogn_message_body1 = r"/074548h5111.32N/00102.04W'086/007/A=000607 !W80! id0ADDE626 -019fpm +0.0rot 5.5dB 3e -4.3kHz gps2x2";
    let expected_body1 = OgnBody {
      timestamp: Utc::today().and_hms(07, 45, 48),
      position: Coordinate {
        x: "511132".parse::<f32>().unwrap() / 10000.0,
        y: "0010204".parse::<f32>().unwrap() / 10000.0,
      },
      ground_track: 86,
      ground_speed: 7.0,
      altitude: 607.0,
      id: "0ADDE626".to_string(),
      climb_rate: -19.0,
      ground_turning_rate: 0.0,
      gps_accuracy: "2x2".to_string(),
    };

    let parsed_body1 = parse_body(ogn_message_body1);

    assert_eq!(parsed_body1.unwrap(), expected_body1);

    // TODO add tests for other possible messages and add support
  }
}
