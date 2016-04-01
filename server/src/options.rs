use clap::{App, Arg};

/// Representation of the application's of all configurable values
#[derive(Debug, Clone)]
pub struct Options {
    /// Host to connect to. Default: `"localhost"`
    pub host: String,
    /// Port on the host to connect to. Default: `8080`
    pub port: u16,
    /// Player size, as defined by [Protocol spec](https://github.com/LoungeCPP/Tatsoryk/wiki/Protocol-spec). Default: `10.0`
    ///
    /// Refer to `Message::Welcome` documentation for details.
    pub player_size: f32,
    /// Bullet size, as defined by [Protocol spec](https://github.com/LoungeCPP/Tatsoryk/wiki/Protocol-spec). Default: `5.0`
    ///
    /// Refer to `Message::Welcome` documentation for details.
    pub bullet_size: f32,
}

impl Options {
    /// Parse `env`-wide command-line arguments into an `Options` instance
    pub fn parse() -> Options {
        static USAGE: &'static str = "[host] 'Host to connect to. Default: localhost'";

        let matches = App::new("tatsoryk-server")
                          .version(env!("CARGO_PKG_VERSION"))
                          .author("nabijaczleweli <nabijaczleweli@gmail.com>,\n\
                                   Cat Plus Plus <piotrlegnica@piotrl.pl>\n\
                                   Lalaland <ethan.steinberg@gmail.com>")
                          .about("Implementation of the server for Tatsoryk")
                          .args_from_usage(USAGE)
                          .arg(Arg::from_usage("[port] 'Port on the host to connect to. \
                                                  Default: 8080'")
                                   .validator(Options::verify_u16))
                          .arg(Arg::from_usage("-p --player-size [player-size] 'Player size. \
                                                Default: 10'")
                                   .validator(Options::verify_positive_f32))
                          .arg(Arg::from_usage("-b --bullet-size [bullet-size] 'Bullet size. \
                                                Default: 5'")
                                   .validator(Options::verify_positive_f32))
                          .get_matches();

        Options {
            host: matches.value_of("host").unwrap_or("127.0.0.1").to_string(),
            port: matches.value_of("port").unwrap_or("8080").parse::<u16>().unwrap(), /* Verified earlier */
            player_size: matches.value_of("player-size").unwrap_or("10.0").parse::<f32>().unwrap(), /* Verified earlier */
            bullet_size: matches.value_of("bullet-size").unwrap_or("5.0").parse::<f32>().unwrap(), /* Verified earlier */
        }
    }

    fn verify_u16(arg: String) -> Result<(), String> {
        match arg[..].parse::<u16>() {
            Ok(_) => Ok(()),
            Err(err) => Err(format!("{:?} is not a 16-bit unsigned integer: {}", arg, err)),
        }
    }

    fn verify_positive_f32(arg: String) -> Result<(), String> {
        match arg[..].parse::<f32>() {
            Ok(0.0) => {
                Err(format!("{:?} is not a 32-bit positive floating-point number: must be nonzero",
                            arg))
            }
            Ok(f) => {
                if f < 0.0 {
                    Err(format!("{:?} is not a 32-bit positive floating-point number: must be \
                                 nonnegative",
                                arg))
                } else {
                    Ok(())
                }
            }
            Err(err) => {
                Err(format!("{:?} is not a 32-bit positive floating-point number: {}",
                            arg,
                            err))
            }
        }
    }
}
