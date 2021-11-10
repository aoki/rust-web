use clap::{arg_enum, App, AppSettings, Arg, SubCommand};

arg_enum! {
    #[derive(Debug)]
    enum Format {
        Csv, Json
    }
}

fn main() {
    // app_from_crate マクロもある
    let opts = App::new(env!("CARGO_PKG_NAME"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        // -s URL | --server URL のオプションを受け取る
        .arg(
            Arg::with_name("SERVER")
                .short("s")
                .long("server")
                .value_name("URL")
                .help("server url")
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("get").about("get logs").arg(
                Arg::with_name("FORMAT")
                    .help("log fromat")
                    .short("f")
                    .long("format")
                    .takes_value(true)
                    // .possible_values(&["csv", "json"])
                    .possible_values(&Format::variants())
                    .case_insensitive(true),
            ),
        );

    let matches = opts.get_matches();
    let server = matches.value_of("SERVER").unwrap_or("localhost:3000");
    match matches.subcommand() {
        ("get", sub_match) => {
            let format: Format = sub_match
                .and_then(|m| m.value_of("FORMAT"))
                .map(|m| m.parse().unwrap())
                .unwrap();
            match format {
                Format::Csv => todo!(),
                Format::Json => todo!(),
            }
        }
        ("post", sub_match) => println!("get: {:?}", sub_match),
        _ => unreachable!(),
    }
}
