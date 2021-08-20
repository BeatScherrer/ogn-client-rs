use ogn_client_rs::APRSClient;
use std::io::Error;

fn main() -> Result<(), Error> {
  let mut client = APRSClient::new("aprs.glidernet.org", 14580);

  let login_message = String::from("user NOCALL pass  -1\n");
  let response = client.send_message(&login_message).unwrap();

  println!("{}", response);

  Ok(())
}
