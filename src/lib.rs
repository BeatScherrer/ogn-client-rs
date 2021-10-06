#[allow(unused_imports)]
#[macro_use]
extern crate pretty_assertions;

use log::{debug, error, info};
use std::fmt::Debug;
use std::io::{BufRead, BufReader, LineWriter, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;

pub mod parser;

#[repr(u16)]
pub enum PORT {
  /// Subscribe to the full feed (note: no filtering but does not require authentication)
  FULLFEED = 10152,
  /// Port on which filtering is supported
  FILTER = 14580,
}

pub struct APRSClient {
  m_reader: BufReader<TcpStream>,
  m_writer: LineWriter<TcpStream>,
  m_callback: Box<dyn Fn(&str) + Send>,
  m_thread: Option<std::thread::JoinHandle<()>>,
  m_terminate: bool,
  m_logged_in: bool,
  m_user: Option<String>,
}

impl APRSClient {
  // ------------------------------------------------------------------------------
  // Public interface
  // ------------------------------------------------------------------------------
  pub fn new(target: &str, port: PORT, callback: Box<dyn Fn(&str) + Send>) -> Arc<Mutex<Self>> {
    // ip addr
    let port = port as u16;
    info!("creating aprs client with target '{}:{}'", target, port);

    let connection = TcpStream::connect((target, port)).unwrap();

    let client = Arc::new(Mutex::new(APRSClient {
      m_writer: LineWriter::new(connection.try_clone().unwrap()),
      m_reader: BufReader::new(connection),
      m_callback: callback,
      m_thread: None,
      m_terminate: false,
      m_logged_in: false,
      m_user: None,
    }));
    // read welcome message
    info!("{}", client.lock().unwrap().read().unwrap());

    // APRSClient::run(client.clone());

    client
  }

  pub fn login(&mut self, login_data: LoginData) -> Result<(), std::io::Error> {
    info!("login with following data: {:?}...", &login_data);

    let login_message = APRSClient::create_aprs_login(&login_data);

    self.send_message(login_message.as_str())?;
    let login_answer = self.read()?;
    debug!("login answer:  {}", login_answer);

    self.m_logged_in = parser::parse_login_answer(&login_answer);

    match self.m_logged_in {
      true => {
        info!("...logged in successfully.");
        self.m_user = Some(String::from(login_data.user_name));
        Ok(())
      }
      false => {
        error!("...failed to log in!");
        Err(std::io::Error::new(
          std::io::ErrorKind::PermissionDenied,
          format!(
            "could not log in with given credentials: {:#?}",
            &login_data
          ),
        ))
      }
    }
  }

  pub fn login_default(&mut self) -> Result<(), std::io::Error> {
    self.login(LoginData::new())
  }

  pub fn run(this: Arc<Mutex<Self>>) {
    info!("starting the client reader thread...");

    let clone = this.clone();

    this.lock().unwrap().m_thread = Some(std::thread::spawn(move || {
      // read the message and pass it to the callback
      let mut lock = clone.lock().unwrap();

      while !lock.m_terminate {
        let message = lock.read().unwrap();
        (lock.m_callback)(&message);
      }
    }));
  }

  pub fn send_position(&mut self, _position: &str) -> Result<(), std::io::Error> {
    // make sure we are logged in
    if let None = self.m_user {
      return Err(std::io::Error::new(
        std::io::ErrorKind::PermissionDenied,
        "permission denied",
      ));
    }

    // create the position message

    // get the timestamp
    // let now = Utc::now();
    // convert time to wonky format
    let now_wonky = "123456";
    let position = "5123.45N/00123.45E";
    let ground_track = 180;
    let ground_speed = 25;
    let altitude = 1000;

    // general aprs message (i.e. before !xx! separator)
    let position_message = format!(
      "{}>OGNAPP:/{}h{}'{}/{}/A={}",
      "beat", now_wonky, position, ground_track, ground_speed, altitude
    );

    self.send_message(&position_message)
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

  pub fn send_status(&self, status_message: &OgnStatusMessage) {
    if self.is_logged_in() {
      // TODO serialize status message and pass along the line
      debug!("{:#?}", status_message);
    } else {
      error!("not logged in, cannot send status message!");
    }
  }

  pub fn set_filter(&mut self, filter_expression: &str) {
    debug!("applying filter: '{}'", filter_expression);
    self
      .send_message(&format!("#filter {}", filter_expression))
      .unwrap();
  }

  // ------------------------------------------------------------------------------
  // Private interface
  // ------------------------------------------------------------------------------
  fn read(&mut self) -> Result<String, std::io::Error> {
    let mut string_buffer = String::new();

    self.m_reader.read_line(&mut string_buffer)?;
    string_buffer = string_buffer.trim_end().to_string();
    debug!("read message: {}", string_buffer);

    Ok(string_buffer)
  }

  fn create_aprs_login(login_data: &LoginData) -> String {
    format!(
      "user {} pass {} vers {} {}",
      login_data.user_name, login_data.pass_code, login_data.app_name, login_data.app_version
    )
  }

  fn is_logged_in(&self) -> bool {
    self.m_logged_in
  }
}

impl Drop for APRSClient {
  fn drop(&mut self) {
    info!("...terminating the aprs client!");
    println!("test");

    self.m_terminate = true;
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
  pub fn new() -> LoginData<'a> {
    Self {
      user_name: "N0CALL",
      pass_code: "-1",
      app_name: env!("CARGO_PKG_NAME"),
      app_version: env!("CARGO_PKG_VERSION"),
    }
  }

  pub fn user_name(mut self, user_name: &'a str) -> LoginData<'a> {
    self.user_name = user_name;
    self
  }

  pub fn pass_code(mut self, pass_code: &'a str) -> LoginData<'a> {
    self.pass_code = pass_code;
    self
  }

  pub fn app_name(mut self, app_name: &'a str) -> LoginData<'a> {
    self.app_name = app_name;
    self
  }

  pub fn app_version(mut self, app_version: &'a str) -> LoginData<'a> {
    self.app_version = app_version;
    self
  }

  pub fn build(&mut self) -> &mut LoginData<'a> {
    self
  }
}

#[derive(Debug, PartialEq)]
pub struct OgnStatusMessage {
  pilot_name: Option<String>,
  manufacturer: Option<String>,
  model: Option<String>,
  make: Option<String>,
  serial_number: Option<String>,
  competition_id: Option<String>,
  competition_class: Option<String>,
  competition_task: Option<String>,
  base_airfield: Option<String>,
  in_case_of_emergency: Option<String>,
  pilot_id: Option<String>,
  hardware: Option<String>,
  software: Option<String>,
}

impl OgnStatusMessage {
  pub fn new() -> Self {
    Self {
      pilot_name: None,
      manufacturer: None,
      model: None,
      make: None,
      serial_number: None,
      competition_id: None,
      competition_class: None,
      competition_task: None,
      base_airfield: None,
      in_case_of_emergency: None,
      pilot_id: None,
      hardware: None,
      software: None,
    }
  }

  pub fn pilot_name(&mut self, pilot_name: &str) -> &mut Self {
    self.pilot_name = Some(String::from(pilot_name));
    self
  }

  pub fn manufacturer(&mut self, manufacturer: &str) -> &mut Self {
    self.manufacturer = Some(String::from(manufacturer));
    self
  }

  pub fn model(mut self, model: &str) -> Self {
    self.model = Some(String::from(model));
    self
  }
  pub fn make(mut self, make: &str) -> Self {
    self.make = Some(String::from(make));
    self
  }
  pub fn serial_number(mut self, serial_number: &str) -> Self {
    self.serial_number = Some(String::from(serial_number));
    self
  }
  pub fn competition_id(mut self, competition_id: &str) -> Self {
    self.competition_id = Some(String::from(competition_id));
    self
  }
  pub fn competition_class(mut self, competition_class: &str) -> Self {
    self.competition_class = Some(String::from(competition_class));
    self
  }
  pub fn competition_task(mut self, competition_task: &str) -> Self {
    self.competition_task = Some(String::from(competition_task));
    self
  }
  pub fn base_airfield(mut self, base_airfield: &str) -> Self {
    self.base_airfield = Some(String::from(base_airfield));
    self
  }
  pub fn in_case_of_emergency(mut self, in_case_of_emergency: &str) -> Self {
    self.in_case_of_emergency = Some(String::from(in_case_of_emergency));
    self
  }
  pub fn pilot_id(mut self, pilot_id: &str) -> Self {
    self.pilot_id = Some(String::from(pilot_id));
    self
  }
  pub fn hardware(mut self, hardware: &str) -> Self {
    self.hardware = Some(String::from(hardware));
    self
  }
  pub fn software(mut self, software: &str) -> Self {
    self.software = Some(String::from(software));
    self
  }
}
