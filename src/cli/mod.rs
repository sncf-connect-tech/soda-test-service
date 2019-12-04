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
      Arg::with_name("verbose")
        .short("-v")
        .long("verbose")
        .multiple(true)
        .help("if you submit [--verbose] you can pin up the logs with all informations, else only that you have configure")
        .takes_value(false)
        .required(false),
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
