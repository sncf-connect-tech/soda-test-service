use clap::{App, Arg, ArgMatches};

fn validate_format(v: String) -> Result<(), String> {
  if v.contains(':') {
    return Ok(());
  }
  Err(String::from("Format must be IP:PORT"))
}

pub fn init<'a>() -> ArgMatches<'a> {
  App::new("HTTP Proxy")
    .arg(
      Arg::with_name("listen")
        .long("listen")
        .help("format : IP:PORT")
        .takes_value(true)
        .validator(validate_format)
        .required(true),
    )
    .arg(
      Arg::with_name("forward")
        .long("forward")
        .help("format : IP:PORT")
        .takes_value(true)
        .validator(validate_format)
        .required(true),
    )
    .arg(
      Arg::with_name("default_logs")
        .long("default_logs")
        .help("format:bool")
        .takes_value(true)
        .validator(validate_format)
        .required(true),
    )
    .arg(
      Arg::with_name("timeout")
        .long("timeout")
        .help("format : DURATION_IN_SECS")
        .takes_value(true)
        .required(true),
    )
    .get_matches()
}
