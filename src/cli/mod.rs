use clap::{Arg, ArgMatches};

pub fn init<'a>() -> ArgMatches<'a> {
    clap::App::new("HTTP Proxy")
        .arg(
            Arg::with_name("listen_addr")
                .takes_value(true)
                .value_name("LISTEN ADDR")
                .index(1)
                .required(true),
        )
        .arg(
            Arg::with_name("listen_port")
                .takes_value(true)
                .value_name("LISTEN PORT")
                .index(2)
                .required(true),
        )
        .arg(
            Arg::with_name("forward_addr")
                .takes_value(true)
                .value_name("FWD ADDR")
                .index(3)
                .required(true),
        )
        .arg(
            Arg::with_name("forward_port")
                .takes_value(true)
                .value_name("FWD PORT")
                .index(4)
                .required(true),
        )
        .get_matches()
}
