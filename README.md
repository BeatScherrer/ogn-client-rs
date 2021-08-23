# ogn-client-rs
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT) ![develop](https://github.com/BeatScherrer/ogn-client-rs/actions/workflows/rust.yml/badge.svg?branch=develop)

This crate provides a Rust client for the [OGN (Open Glider Network)](http://wiki.glidernet.org/). A default parser is provided which parses the OGN messages (which are in an extended APRS format) to a corresponding Rust data structure.

The default parser should be fine for most use cases. To separate the library from users' libraries the resulting data structure can be converted if needed.

In case a parse for your custom data structure is desired simply the `Parse` trait.

## Usage
TODO add usage