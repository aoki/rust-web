mod api_client;

use api_client::ApiClient;
use clap::{arg_enum, App, AppSettings, Arg, SubCommand};
use reqwest::blocking::Client;
use std::io::{self, Write};

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
        .subcommand(SubCommand::with_name("post").about("post logs, taking input from stdin"))
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
    let server: String = matches
        .value_of("SERVER")
        .unwrap_or("localhost:3000")
        .into();
    let client = Client::new();
    let api_client = ApiClient { server, client };
    match matches.subcommand() {
        ("get", sub_match) => {
            let format: Format = sub_match
                .and_then(|m| m.value_of("FORMAT"))
                .map(|m| m.parse().unwrap())
                .unwrap();
            match format {
                Format::Csv => do_get_csv(&api_client),
                Format::Json => do_get_json(&api_client),
            }
        }
        ("post", _) => do_post_csv(&api_client),
        _ => unreachable!(),
    }
}

fn do_post_csv(api_client: &ApiClient) {
    let reader = csv::Reader::from_reader(io::stdin());
    for log in reader.into_deserialize::<api::logs::post::Request>() {
        let log = match log {
            Ok(log) => log,
            Err(e) => {
                eprintln!("[WARN] failed to parse a line, skipping: {}", e);
                continue;
            }
        };
        api_client.psot_logs(&log).expect("api request failed");
    }
}

fn do_get_json(api_client: &ApiClient) {
    let res = api_client.get_logs().expect("api request failed");
    let json_str = serde_json::to_string(&res).unwrap();
    println!("{}", json_str);
}

fn do_get_csv(api_client: &ApiClient) {
    let out = io::stdout();
    let mut out = out.lock();
    api_client.get_csv(&mut out).expect("api request failed");
    out.flush().unwrap();

    // https://rust-lang-nursery.github.io/rust-cookbook/encoding/csv.html#serialize-records-to-csv
    // let mut wtr = csv::Writer::from_writer(io::stdout());
    // wtr.write_record(&["UA", "ResponseTime", "TimeStamp"])
    //     .unwrap();
    // for r in res.0 {
    //     wtr.serialize(r).unwrap();
    // }
    // wtr.flush().unwrap();
}
