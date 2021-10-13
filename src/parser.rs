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
  ground_turning_rate: Option<f32>,
  climb_rate: Option<f32>,
  altitude: f32,
  ground_track: u16,
  gps_accuracy: Option<String>,
  id: Option<String>,
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
  // check if m

  // parse aprs part
  let aprs_regex = Regex::new(
    r"/(?P<time>\d{6})h(?P<lat>.*)[NS].(?P<lon>.*)[EW].(?P<ground_track>\d{3})/(?P<ground_speed>\d{3})/A=(?P<altitude>\d{6})",
  )
  .expect("bad aprs regex");

  let captures = aprs_regex.captures(body);
  if let None = captures {
    error!("error while parsing aprs part of body! {}", body);
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
  let ground_track: u16 = captures.name("ground_track").unwrap().as_str().parse().unwrap();

  // ground speed
  let ground_speed: f32 = captures.name("ground_speed").unwrap().as_str().parse().unwrap();

  // altitude
  let altitude: f32 = captures.name("altitude").unwrap().as_str().parse().unwrap();

  // ------------------------------------------------------------------------------
  // parse ogn part
  // ------------------------------------------------------------------------------
  let id = parse_id(body);
  let climb_rate = parse_climb_rate(body);
  let rotation_rate = parse_rotation_rate(body);
  let gps_accuracy = parse_gps_accuracy(body);

  // assemble the message
  Some(OgnBody {
    timestamp: timestamp,
    position: Coordinate {
      x: lat.parse::<f32>().unwrap() / 10000.0,
      y: lon.parse::<f32>().unwrap() / 10000.0,
    },
    ground_speed: ground_speed,
    ground_turning_rate: rotation_rate,
    climb_rate: climb_rate,
    altitude: altitude,
    ground_track: ground_track,
    gps_accuracy: gps_accuracy,
    id: id,
  })
}

pub fn parse_login_answer(login_answer: &str) -> bool {
  let re = Regex::new(r"^# logresp (\w+) (verified)").unwrap();

  match re.find(login_answer) {
    Some(_) => true,
    _ => false,
  }
}

fn parse_id(body: &str) -> Option<String> {
  let regex = Regex::new(r"id(?P<id>\w{8})").unwrap();
  let captures = regex.captures(body);

  if let Some(v) = captures {
    Some(v.name("id").unwrap().as_str().parse().unwrap())
  } else {
    None
  }
}

fn parse_climb_rate(body: &str) -> Option<f32> {
  let regex = Regex::new(r"(?P<climb>[+-].*)fpm").unwrap();
  let captures = regex.captures(body);
  if let Some(v) = captures {
     Some(v.name("climb").unwrap().as_str().parse().unwrap())
  } else {
    None
  }
}

fn parse_rotation_rate(body: &str) -> Option<f32> {
  let regex = Regex::new(r"(?P<rot>[+-][0-9.]+)rot").unwrap();
  let captures = regex.captures(body);
  if let Some(v) = captures {
    Some(v.name("rot").unwrap().as_str().parse().unwrap())
  } else {
    None
  }
}

fn parse_gps_accuracy(body: &str) -> Option<String> {
  let regex = Regex::new(r"gps(?P<accuracy>\dx\d)").unwrap();
  let captures = regex.captures(body);
  if let Some(v) = captures {
    Some(v.name("accuracy").unwrap().as_str().parse().unwrap())
  } else {
    None
  }

}


#[cfg(test)]
mod tests {
  use super::*;

  use std::sync::Once;

  static INIT: Once = Once::new();

  fn setup() {
    INIT.call_once(|| {
      log4rs::init_file("logger_config.yaml", Default::default()).unwrap();
    });
  }

  #[test]
  fn parse_full_message() {
    setup();

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
      climb_rate: Some(0.0),
      ground_speed: 0.0,
      ground_turning_rate: Some(-4.3),
      ground_track: 232,
      gps_accuracy: Some("3x5".to_string()),
      id: Some("3782149C".to_string()),
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
    setup();

    assert!(parse_login_answer(r"# logresp user verified, server GLIDERN2") == true);

    assert!(parse_login_answer(r"# logresp user unverified, server GLIDERN1") == false);
    assert!(parse_login_answer("random string") == false);
  }

  #[test]
  fn parse_message_header_test() {
    setup();

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
    setup();

    // Case 1
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
      id: Some("0ADDE626".to_string()),
      climb_rate: Some(-19.0),
      ground_turning_rate: Some(0.0),
      gps_accuracy: Some("2x2".to_string()),
    };
    let parsed_body1 = parse_body(ogn_message_body1);
    assert_eq!(parsed_body1.unwrap(), expected_body1);

    // case 2
    let ogn_message_body2 = r"/200746h5008.11N\00839.28En000/000/A=001280 !W51! id3ED0077D -019fpm +0.0rot 0.2dB 4e -6.9kHz gps2x4";
    let expected_body2 = OgnBody {
      timestamp: Utc::today().and_hms(20, 07, 46),
      position: Coordinate {
        x: "500811".parse::<f32>().unwrap() / 10000.0,
        y: "083928".parse::<f32>().unwrap() / 10000.0,
      },
      ground_track: 000,
      ground_speed: 000.0,
      altitude: 1280.0,
      id: Some("3ED0077D".to_string()),
      climb_rate: Some(-19.0),
      ground_turning_rate: Some(0.0),
      gps_accuracy: Some("2x4".to_string()),
    };
    let parsed_body2 = parse_body(ogn_message_body2);
    assert_eq!(parsed_body2.unwrap(), expected_body2);

    // case 3
    let ogn_message_body3 =
      r"/162405h4925.73N/01706.72E'161/066/A=000790 !W52! id2022449E +003fpm gps5x3";
    let expected_body3 = OgnBody {
      timestamp: Utc::today().and_hms(16, 24, 05),
      position: Coordinate {
        x: "492573".parse::<f32>().unwrap() / 10000.0,
        y: "0170672".parse::<f32>().unwrap() / 10000.0,
      },
      ground_track: 161,
      ground_speed: 66.0,
      altitude: 790.0,
      id: Some("2022449E".to_string()),
      climb_rate: Some(3.0),
      gps_accuracy: Some("5x3".to_string()),
      ground_turning_rate: None,
    };
    let parsed_body3 = parse_body(ogn_message_body3);
    assert_eq!(parsed_body3.unwrap(), expected_body3);

    // case 4
    let ogn_message_body4 = r"/164425h5115.68N/00005.56Wz000/001/A=000614 !W25! id0308A689 +0fpm FNT10 22.0dB +58.8kHz 2e";
    let expected_body4 = OgnBody {
      timestamp: Utc::today().and_hms(16, 44, 25),
      position: Coordinate {
        x: "511568".parse::<f32>().unwrap() / 10000.0,
        y: "000556".parse::<f32>().unwrap() / 10000.0,
      },
      ground_track: 000,
      ground_speed: 1.0,
      altitude: 614.0,
      id: Some("0308A689".to_string()),
      climb_rate: Some(0.0),
      gps_accuracy: None,
      ground_turning_rate: None,
    };

    let parsed_body4 = parse_body(ogn_message_body4).unwrap();
    assert_eq!(parsed_body4, expected_body4);
  }
}
