#[allow(unused_imports)]
#[macro_use]
extern crate pretty_assertions;

use chrono::Timelike;
use chrono::Utc;
use log::{debug, error, info};
use std::fmt::Debug;
use std::io::{BufRead, BufReader, LineWriter, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;

pub mod parser;

#[repr(u16)]
#[derive(Debug, Clone, PartialEq)]
pub enum PORT {
  /// Subscribe to the full feed (note: no filtering but does not require authentication)
  FULLFEED = 10152,
  /// Port on which filtering is supported
  FILTER = 14580,
}

pub struct APRSClient {
  m_target: String,
  m_port: PORT,
  m_reader: Option<BufReader<TcpStream>>,
  m_writer: Option<LineWriter<TcpStream>>,
  m_callback: Box<dyn Fn(&str) + Send>,
  m_thread: Option<std::thread::JoinHandle<()>>,
  m_terminate: bool,
  m_logged_in: bool,
  m_user: Option<String>,
  m_is_connected: bool,
}

impl APRSClient {
  // ------------------------------------------------------------------------------
  // Public interface
  // ------------------------------------------------------------------------------
  pub fn new(target: &str, port: PORT, callback: Box<dyn Fn(&str) + Send>) -> Arc<Mutex<Self>> {
    // ip addr
    info!("creating aprs client with target '{}:{:#?}'", target, port);

    let client = Arc::new(Mutex::new(APRSClient {
      m_port: port,
      m_target: target.to_string(),
      m_writer: None,
      m_reader: None,
      m_callback: callback,
      m_thread: None,
      m_terminate: false,
      m_logged_in: false,
      m_user: None,
      m_is_connected: false,
    }));

    // lock scope
    {
      let mut locked_client = client.lock().unwrap();

      // try to connect to the server
      let _ = locked_client.connect();
    }

    client
  }

  pub fn is_connected(&self) -> bool {
    self.m_is_connected
  }

  pub fn connect(&mut self) -> Result<(), std::io::Error> {
    let target = format!("{}:{}", self.m_target, self.m_port.clone() as u16);

    info!("trying to connect to {}...", target);

    let connection = TcpStream::connect(target);

    match connection {
      Ok(_) => info!("...connection successfully established"),
      Err(err) => {
        error!("{}", err);
        return Err(err);
      }
    };

    let connection = connection.unwrap();

    self.m_writer = Some(LineWriter::new(connection.try_clone().unwrap()));
    self.m_reader = Some(BufReader::new(connection));

    self.m_is_connected = true;

    // read welcome message from server
    info!("{}", self.read().unwrap());

    Ok(())
  }

  pub fn login(&mut self, login_data: &LoginData) -> Result<(), std::io::Error> {
    info!("logging in with:\n{:#?}", &login_data);

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
    if !self.is_logged_in() {
      return Err(std::io::Error::new(
        std::io::ErrorKind::NotConnected,
        "client is not connected to the server, make sure to connect first",
      ));
    }

    self.login(&LoginData::new())
  }

  pub fn is_logged_in(&self) -> bool {
    self.m_logged_in
  }

  pub fn run(this: Arc<Mutex<Self>>) {
    info!("starting the client reader thread...");

    //
    let mut lock = this.lock().unwrap();
    if !lock.m_is_connected {
      info!("currently not connected, trying to connect...");
      lock.connect().unwrap();
    }

    let clone = this.clone();

    lock.m_thread = Some(std::thread::spawn(move || {
      // read the message and pass it to the callback
      let mut lock = clone.lock().unwrap();

      while !lock.m_terminate {
        let message = lock.read().unwrap();
        (lock.m_callback)(&message);
        // std::thread::sleep(std::time::Duration::from_millis(10));
        std::thread::sleep(std::time::Duration::from_secs(1));
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

    // get the timestamp
    let now = Utc::now();
    let mut time_string = String::new();
    time_string.push_str(&now.minute().to_string());
    time_string.push_str(&now.hour().to_string());
    time_string.push_str(&now.second().to_string());

    // convert time to wonky format
    let position = "5123.45N/00123.45E";
    let ground_track = 180;
    let ground_speed = 25;
    let altitude = 1000;

    // general aprs message (i.e. before ogn extras)
    let position_message = format!(
      "{}>OGNAPP:/{}h{}'{}/{}/A={}",
      self.m_user.as_ref().unwrap(),
      time_string,
      position,
      ground_track,
      ground_speed,
      altitude
    );

    self.send_message(&position_message)
  }

  pub fn send_status(&self, status_message: &OgnStatusMessage) {
    if self.is_logged_in() {
      // TODO serialize status message and pass along the line
      debug!("{:#?}", status_message);
    } else {
      error!("not logged in, cannot send status message!");
    }
  }

  pub fn set_filter(&mut self, filter_expression: &str) -> Result<(), std::io::Error> {
    if self.m_port != PORT::FILTER {
      error!("connected to fullfeed port, cannot set a filter");
      return Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "cannot set filter on fullfeed port",
      ));
    }

    debug!("applying filter: '{}'", filter_expression);
    self.send_message(&format!("#filter {}", filter_expression))
  }

  // ------------------------------------------------------------------------------
  // Private interface
  // ------------------------------------------------------------------------------
  fn read(&mut self) -> Result<String, std::io::Error> {
    let mut string_buffer = String::new();

    //TODO add timeout and disconnect

    self
      .m_reader
      .as_mut()
      .unwrap()
      .read_line(&mut string_buffer)?;
    string_buffer = string_buffer.trim_end().to_string();
    debug!("read message: {}", string_buffer);

    Ok(string_buffer)
  }

  fn send_message(&mut self, message: &str) -> Result<(), std::io::Error> {
    // check if we are connected
    if !self.m_is_connected {
      error!("currently not connected, cannot send message");
      return Err(std::io::Error::new(
        std::io::ErrorKind::NotConnected,
        "not connected, cannot send message",
      ));
    }

    let mut full_message = String::new();
    debug!("sending message: '{}'", message);

    full_message.push_str(message);
    full_message.push_str("\r\n");
    self
      .m_writer
      .as_mut()
      .unwrap()
      .write_all(full_message.as_bytes())?;
    self.m_writer.as_mut().unwrap().flush()?;

    Ok(())
  }

  // fn send_heart_beat(&mut self) {
  //   self.send_message("#keepalive").unwrap();
  // }

  fn create_aprs_login(login_data: &LoginData) -> String {
    format!(
      "user {} pass {} vers {} {}",
      login_data.user_name, login_data.pass_code, login_data.app_name, login_data.app_version
    )
  }
}

impl Drop for APRSClient {
  fn drop(&mut self) {
    info!("...terminating the aprs client!");

    self.m_terminate = true;
  }
}

#[derive(Debug)]
pub struct LoginData {
  pub user_name: &'static str,
  pub pass_code: &'static str,
  pub app_name: &'static str,
  pub app_version: &'static str,
}

impl LoginData {
  pub fn new() -> LoginData {
    Self {
      user_name: "N0CALL",
      pass_code: "-1",
      app_name: env!("CARGO_PKG_NAME"),
      app_version: env!("CARGO_PKG_VERSION"),
    }
  }

  pub fn user_name(mut self, user_name: &'static str) -> LoginData {
    self.user_name = user_name;
    self
  }

  pub fn pass_code(mut self, pass_code: &'static str) -> LoginData {
    self.pass_code = pass_code;
    self
  }

  pub fn app_name(mut self, app_name: &'static str) -> LoginData {
    self.app_name = app_name;
    self
  }

  pub fn app_version(mut self, app_version: &'static str) -> LoginData {
    self.app_version = app_version;
    self
  }

  pub fn build(self) -> LoginData {
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

// {
//   "/z",  //  0 = ?
//   "/'",  //  1 = (moto-)glider    (most frequent)
//   "/'",  //  2 = tow plane        (often)
//   "/X",  //  3 = helicopter       (often)
//   "/g" , //  4 = parachute        (rare but seen - often mixed with drop plane)
//   "\\^", //  5 = drop plane       (seen)
//   "/g" , //  6 = hang-glider      (rare but seen)
//   "/g" , //  7 = para-glider      (rare but seen)
//   "\\^", //  8 = powered aircraft (often)
//   "/^",  //  9 = jet aircraft     (rare but seen)
//   "/z",  //  A = UFO              (people set for fun)
//   "/O",  //  B = balloon          (seen once)
//   "/O",  //  C = airship          (seen once)
//   "/'",  //  D = UAV              (drones, can become very common)
//   "/z",  //  E = ground support   (ground vehicles at airfields)
//   "\\n"  //  F = static object    (ground relay ?)
// } ;
