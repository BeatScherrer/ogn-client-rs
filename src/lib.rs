use std::io::BufRead;
use std::io::Read;
use std::io::{BufReader};
use std::net::TcpStream;
use log::{debug, info};

pub struct APRSClient<'a> {
  m_connection: TcpStream,
  m_buffer: &'a mut[u8; 128]
}

impl<'a> APRSClient<'a> {
  pub fn new(target: &str, port: u16) -> Self {
    // ip addr
    info!("creating aprs client with target '{}:{}'", target, port);

    APRSClient {
      m_connection: TcpStream::connect((target, port)).unwrap(),
      m_buffer: &mut[0; 128],
    }
  }

  pub fn run(&self) {
    info!("starting the client...");

    // create the buffer reader first which handles read
    let mut reader = BufReader::new(&self.m_connection);

    // create buffer
    let mut buffer: String;

    info!("starting the listening loop...");
    loop {
      // clear the buffer
      buffer.clear();

      self.send_heart_beat();

      let result = reader.read_line(&mut buffer);
    }
  }

  /// Send bytes and return the answer
  pub fn send_message(&mut self, message: &str) -> Result<String, std::io::Error> {
    let byte_message = message.as_bytes();

    self.m_connection.write(byte_message)?;

    // receive the answer
    let mut buffer = [0, 128];
    let bytes_read = self.m_connection.read(&mut buffer)?;

    // convert bytes read to string slice
    Ok(
      std::str::from_utf8(&buffer[..bytes_read])
        .unwrap()
        .to_string(),
    )
  }

  pub fn send_heart_beat(&self) {
    self.send_message("#keepalive\n");
    debug!("sent heartbeat");
  }

  fn read(&self) -> Result<String, std::io::Error> {
    let mut reader = BufferedReader::new(m_connection);
    self.m_connection.read(&mut self.m_buffer);

    Ok(String::from(""))
  }
  // TODO maybe split receiving and sending
}

impl<'a> Drop for APRSClient<'a>{
  fn drop(&mut self) {
    info!("...terminating the aprs client!");
  }
}

struct LoginData<'a> {
  pub user_name: &'a str,
  pub pass_code: &'a str,
  pub app_name: &'a str,
  pub app_version: &'a str,
}

fn create_aprs_login(login_data: LoginData) -> String {
  format!(
    "user {} pass {} vers {} {}\n",
    login_data.user_name, login_data.pass_code, login_data.app_name, login_data.app_version
  )
}
