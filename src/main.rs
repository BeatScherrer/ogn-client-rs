use ogn_client_rs::APRSClient;
use std::io::Error;

fn main() -> Result<(), Error> {
  let mut client = APRSClient::new("aprs.glidernet.org", 14580);

  let login_message = String::from("user BEAT pass -1 vers RustClient filter r/33/-97/200 t/n");
  let response = client.send_message(&login_message);

  println!("{:?}", response);

  Ok(())
}
