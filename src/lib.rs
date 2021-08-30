use log::{debug, info};
use std::io::{BufRead, BufReader, LineWriter, Write};
use std::net::TcpStream;

mod parser;
use parser::Parse;

// impl OgnStatusMessage {
// pub fn new() -> Self {
//   OgnStatusMessage {
//     m_pilot_name: None,
//     m_manufacturer: None,
//     m_model: None,
//     m_type: None,
//     m_serial_number: None,
//     m_competition_id: None,
//     m_competition_class: None,
//     m_competition_task: None,
//     m_base_airfield: None,
//     m_in_case_of_emergency: None,
//     m_pilot_id: None,
//     m_hardware: None,
//     m_software: None,
//   }
// }
// }

#[repr(u16)]
pub enum PORT {
  FULLFEED = 10152,
  FILTER = 14580,
}

pub struct APRSClient {
  m_reader: BufReader<TcpStream>,
  m_writer: LineWriter<TcpStream>,
  // m_parser: Box<dyn Parse>,
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
