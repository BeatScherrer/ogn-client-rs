use std::io::{Write, BufReader, LineWriter, BufRead};
use std::net::TcpStream;
use log::{debug, info};

pub struct APRSClient {
  m_reader: BufReader<TcpStream>,
  m_writer: LineWriter<TcpStream>
  // m_buffer: &'a mut[u8; 128]
}

impl APRSClient {
  pub fn new(target: &str, port: u16) -> Self {
    // ip addr
    info!("creating aprs client with target '{}:{}'", target, port);

    let connection = TcpStream::connect((target, port)).unwrap();

    APRSClient {
      m_writer: LineWriter::new(connection.try_clone().unwrap()),
      m_reader: BufReader::new(connection)
      // m_buffer: &mut[0; 128],
    }
  }

  pub fn run(&mut self) {
    info!("starting the client...");

    loop {
      self.send_heart_beat();

      let result = self.read().unwrap();
      info!("{:?}", result);
    }
  }

  /// Send bytes and return the answer
  pub fn send_message(&mut self, message: &str) -> Result<(), std::io::Error> {
    let mut full_message = String::new();
    full_message.push_str(message);
    full_message.push_str("\n");

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

    Ok(string_buffer)
  }
}

impl Drop for APRSClient{
  fn drop(&mut self) {
    info!("...terminating the aprs client!");
  }
}

pub struct LoginData {
  pub user_name: String,
  pub pass_code: String,
  pub app_name: String,
  pub app_version: String,
}

impl LoginData {
  pub fn new(user_name: &str, pass_code: Option<&str>, app_name: &str, app_version: Option<&str>) -> LoginData {

    Self{
      user_name: user_name.to_string(),
      pass_code: pass_code.get_or_insert("-1").to_string(),
      app_name: app_name.to_string(),
      app_version: env!("CARGO_PKG_VERSION").to_string()
    }
  }
}

fn create_aprs_login(mut login_data: LoginData) -> String {
  login_data.pass_code.get_or_insert(String::from("-1"));

  format!(
    "user {} pass {} vers {} {}",
    login_data.user_name, login_data.pass_code.unwrap(), login_data.app_name, login_data.app_version
  )
}
