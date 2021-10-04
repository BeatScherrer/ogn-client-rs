# ogn-client-rs
[![License: LGPL v3](https://img.shields.io/badge/License-LGPL%20v3-blue.svg)](https://www.gnu.org/licenses/lgpl-3.0) ![develop](https://github.com/BeatScherrer/ogn-client-rs/actions/workflows/rust.yml/badge.svg?branch=develop)

This crate provides a Rust client for the [OGN (Open Glider Network)](http://wiki.glidernet.org/). A default parser is provided which parses the OGN messages (which are in an extended APRS format) to a corresponding Rust data structure.

The default parser should be fine for most use cases. To separate the library from users' libraries the resulting data structure can be converted if needed.

In case a parse for your custom data structure is desired simply the `Parse` trait.

## Usage
Messages can be read by passing a callback to the client like in the following simple example:
```rust
use ogn_client_rs::{APRSClient, PORT};

fn main() {

  // create a closure which takes a &str as parameter. Can also be a regular function.
  let callback = |message: &str| {
    println!("from callback: {}", message);
  }

  let client = APRSClient::new("aprs.glidernet.org", PORT::FULLLFEED, Box::new(callback));

  // keep the client alive for 5s
  std::thread::sleep(std::time::Duration::from_secs(5));

  // client runs out of scope and gets destroyed
}
```



## TODO:
### Library
- check for login status
- check for connection status
- add send status
- add passcode generation
- add documentation for library

### parsing
- Make sure the lat/long conversion to coordinate is correct
- create an example with a parser

# Further Details
## aprs-is notes:
- [ogn aprs protocol](http://wiki.glidernet.org/wiki:ogn-flavoured-aprs)
- [ogn-wiki](http://wiki.glidernet.org/aprs-interaction-examples)
- constant information should only be sent every 5 minutes
- After every 20s a heartbeat is sent from the server, try to reconnect
after 1min of not receiving the heartbeat

an example of a ogn message looks as follows:
```
OGN82149C>OGNTRK,qAS,OxfBarton:/130208h5145.95N/00111.50W'232/000/A=000295 !W33! id3782149C +000fpm -4.3rot FL000.00 55.0dB 0e -3.7kHz gps3x5
```

where the header `OGN82149C>OGNTRK,qAS,OxfBarton` and message `/130208h5145.95N/00111.50W'232/000/A=000295 !W52! id3782149C +000fpm -4.3rot FL000.00 55.0dB 0e -3.7kHz gps3x5`
are seperated with a `:`. The fields until `!W52!`, where 5 is the third decimal digit of latitude minutes and 2 is the added digit of longitude minutes, are pure APRS format and after are "comments" which carry ogn specific extra information.

For the processing of the received messages a callback approach is used. This mitigates the responsibility of the
client which should only be responsible for receiving and sending the data.
