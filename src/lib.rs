use log::{debug, info};
use std::fmt::Debug;
use std::io::{BufRead, BufReader, LineWriter, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;

#[repr(u16)]
pub enum PORT {
  FULLFEED = 10152,
  FILTER = 14580,
}

pub struct APRSClient {
  m_reader: BufReader<TcpStream>,
  m_writer: LineWriter<TcpStream>,
  m_callback: Box<dyn Fn(&str) + Send>,
  m_thread: Option<std::thread::JoinHandle<()>>,
}

impl APRSClient {
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
    }));

    APRSClient::run(client.clone());

    client
  }

  pub fn login(&mut self, login_data: LoginData) -> Result<(), std::io::Error> {
    info!("login with following data: {:?}", &login_data);

    let login_message = APRSClient::create_aprs_login(login_data);

    self.send_message(login_message.as_str())?;
    info!("login answer:  {}", self.read()?);

    Ok(())
  }

  pub fn login_default(&mut self) -> Result<(), std::io::Error> {
    self.login(LoginData::new())
  }

  fn run(this: Arc<Mutex<Self>>) {
    info!("starting the client...");

    let clone = this.clone();

    this.lock().unwrap().m_thread = Some(std::thread::spawn(move || {
      // read the message and pass it to the callback
      let message = clone.lock().unwrap().read().unwrap();
      (clone.lock().unwrap().m_callback)(&message);
    }));
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

  pub fn set_filter(&mut self, filter_expression: &str) {
    debug!("applying filter: '{}'", filter_expression);
    self
      .send_message(&format!("#filter {}", filter_expression))
      .unwrap();
  }

  fn read(&mut self) -> Result<String, std::io::Error> {
    let mut string_buffer = String::new();

    self.m_reader.read_line(&mut string_buffer)?;
    string_buffer = string_buffer.trim_end().to_string();

    Ok(string_buffer)
  }

  fn create_aprs_login(login_data: LoginData) -> String {
    format!(
      "user {} pass {} vers {} {}",
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
