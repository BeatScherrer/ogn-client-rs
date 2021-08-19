use std::io::Error;
use std::io::Read;
use std::io::Write;
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

fn handle_client(stream: TcpStream) {
  println!("received {:?}", stream);
}

fn main() -> Result<(), Error> {
  println!("Hello, world!");

  let remote = "aprs.glidernet.org:14580";

  let client = APRSClient::new("aprs.glidernet.org", 14580);

  if let Ok(mut stream) = TcpStream::connect(&remote) {
    println!("Connected to the server '{}'!", &remote);

    // assign buffer to be reused
    let mut buffer = [0, 128];

    let x = 4;

    // send initial message
    /*
    note: m/10 means as much as:
    give me all the positions in a 10km of the position I am going to report you
    */
    let login_message = String::from("user BEAT pass -1 vers RustClient filter r/33/-97/200 t/n");
    let mut bytes_written = stream.write(login_message.as_bytes()).unwrap();
    println!("wrote {:?} bytes", bytes_written);

    // read results to buffer
    let mut bytes_read = stream.read(&mut buffer[..])?;
    let message = std::str::from_utf8(&buffer[..bytes_read]);
    // print buffer
    println!("received {} bytes: \n{:?}", bytes_read, message);

    // report the position
    let position_message = r#"OGN123456>OGNAPP:/123456h5123.45N/00123.45E'180/025/A=001000 !W66! id07123456 +100fpm +1.0rot FL011.00 gps4x5"#;
    bytes_written = stream.write(position_message.as_bytes()).unwrap();
    println!("wrote {:?} bytes", bytes_written);

    // read the response
    // clear the buffer first
    bytes_read = stream.read(&mut buffer[..])?;
    let response = std::str::from_utf8(&buffer[..bytes_read]);
    println!("{:?}", response);
  } else {
    println!("Couldn't connect to server...");
  }

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

struct APRSClient {
  m_socket: std::net::SocketAddr,
  // m_socket: std::net::IpAddr,
  // m_filter: String,
  // m_connection: TcpStream,
  m_connection: TcpStream,
}

impl APRSClient {
  pub fn new(ip: std::net::IpAddr, port: u16) -> Self {
    APRSClient {
      m_socket: std::net::SocketAddr::from((ip, port)),
      m_connection: TcpStream::connect(m_socket).unwrap(),
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
}
