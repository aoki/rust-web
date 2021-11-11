# API Cli

```bash
cargo run -- get -f csv
cat ../test.csv | cargo run -- post
```

```
cli 0.1.0



USAGE:
    cli [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -s, --server <URL>    server url

SUBCOMMANDS:
    get     get logs
    help    Prints this message or the help of the given subcommand(s)
    post    post logs, taking input from stdin
```
