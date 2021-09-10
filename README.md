# ogn-client-rs
[![License: LGPL v3](https://img.shields.io/badge/License-LGPL%20v3-blue.svg)](https://www.gnu.org/licenses/lgpl-3.0) ![develop](https://github.com/BeatScherrer/ogn-client-rs/actions/workflows/rust.yml/badge.svg?branch=develop)

This crate provides a Rust client for the [OGN (Open Glider Network)](http://wiki.glidernet.org/). A default parser is provided which parses the OGN messages (which are in an extended APRS format) to a corresponding Rust data structure.

The default parser should be fine for most use cases. To separate the library from users' libraries the resulting data structure can be converted if needed.

In case a parse for your custom data structure is desired simply the `Parse` trait.

## Usage
TODO add usage


## aprs-is notes:
- constant information should only be sent every 5 minutes
[ogn-wiki](http://wiki.glidernet.org/aprs-interaction-examples)
- After every 20s a heartbeat is sent from the server, try to reconnect
after 1min of not receiving the heartbeat

an example of a ogn message looks as follows:
```
OGN82149C>OGNTRK,qAS,OxfBarton:/130208h5145.95N/00111.50W'232/000/A=000295 !W33! id3782149C +000fpm -4.3rot FL000.00 55.0dB 0e -3.7kHz gps3x5
```

where the header `OGN82149C>OGNTRK,qAS,OxfBarton` and message `/130208h5145.95N/00111.50W'232/000/A=000295 !W33! id3782149C +000fpm -4.3rot FL000.00 55.0dB 0e -3.7kHz gps3x5`
are seperated with a `:`. The fields until `!W33!` are pure APRS format and after are "comments" which carry ogn specific extra information

For the processing of the received messages a callback approach is used. This mitigates the responsibility of the
client which should only be responsible for receiving and sending the data.


## TODO:
### Library
- Make reading asynchronous
- check for login status
- check for connection status
- add send status
- add passcode generation

### parsing
- Make sure the lat/long conversion to coordinate is correct
- create an example with a parser

