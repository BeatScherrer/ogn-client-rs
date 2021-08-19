use std::io::Error;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;

fn main() -> Result<(), Error> {
  let mut client = APRSClient::new("aprs.glidernet.org", 14580);

  let login_message = String::from("user BEAT pass -1 vers RustClient filter r/33/-97/200 t/n");
  let response = client.send_message(&login_message);

  println!("{:?}", response);

  Ok(())
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

// TODO move to library
struct APRSClient {
  m_connection: TcpStream,
}

impl APRSClient {
  pub fn new(target: &str, port: u16) -> Self {
    // ip addr
    APRSClient {
      m_connection: TcpStream::connect((target, port)).unwrap(),
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
  // TODO maybe split receiving and sending
}
