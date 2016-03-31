use clap::{App, Arg};

/// Representation of the application's of all configurable values
#[derive(Debug, Clone, Hash)]
pub struct Options {
    /// Host to connect to. Default: `"localhost"`
    pub host: String,
    /// Port on the host to connect to. Default: `8080`
    pub port: u16,
}

impl Options {
    /// Parse `env`-wide command-line arguments into an `Options` instance
    pub fn parse() -> Options {
        static USAGE: &'static str = "[host] 'Host to connect to. Default: localhost'";

        let matches = App::new("tatsoryk-server")
                          .version(env!("CARGO_PKG_VERSION"))
                          .author("nabijaczleweli <nabijaczleweli@gmail.com>,\n\
                                   Cat Plus Plus <piotrlegnica@piotrl.pl>")
                          .about("Implementation of the server for Tatsoryk")
                          .args_from_usage(USAGE)
                          .arg(Arg::from_usage("[port] 'Port on the host to connect to. \
                                                  Default: 8080'")
                                   .validator(Options::verify_u16))
                          .get_matches();

        Options {
            host: matches.value_of("host").unwrap_or("127.0.0.1").to_string(),
            port: matches.value_of("port").unwrap_or("8080").parse::<u16>().unwrap(), /* Verified earlier */
        }
    }

    fn verify_u16(arg: String) -> Result<(), String> {
        match arg[..].parse::<u16>() {
            Ok(_) => Ok(()),
            Err(err) => Err(format!("{:?} is not a 16-bit unsigned integer: {}", arg, err)),
        }
    }
}
