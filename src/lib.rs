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

    // create the buffer reader first which handles readline
    let mut buffer = String::new();

    info!("starting the listening loop...");
    loop {
      self.send_heart_beat();

      let result = self.m_reader.read_line(&mut buffer);
      debug!("{:?}", result);
    }
  }

  /// Send bytes and return the answer
  pub fn send_message(&mut self, message: &str) -> Result<String, std::io::Error> {
    self.m_writer.write_all(message.as_bytes())?;
    self.m_writer.flush()?;

    self.read()
  }

  pub fn send_heart_beat(&mut self) {
    self.send_message("#keepalive\n").unwrap();
    debug!("sent heartbeat");
  }

  fn read(&mut self) -> Result<String, std::io::Error> {
    
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

struct LoginData {
  pub user_name: String,
  pub pass_code: Option<String>,
  pub app_name: String,
  pub app_version: String,
}

fn create_aprs_login(mut login_data: LoginData) -> String {
  login_data.pass_code.get_or_insert(String::from("-1"));

  format!(
    "user {} pass {} vers {} {}\n",
    login_data.user_name, login_data.pass_code.unwrap(), login_data.app_name, login_data.app_version
  )
}
