# spring4shell-scanner

Network scanner based on Tokio async runtime for detecting the spring4shell
vulnerability (CVE-2022-22965). Currently GET and POST request are checked.
The scanner will read target endpoints from stdin and takes the optional number
of tasks via cli parameter (default is 10).

The detection method is based on the curl command posted by RandoriAttack:

- https://twitter.com/RandoriAttack/status/1509298490106593283


## Run

```sh
spring4shell-scanner 0.8.0
Network based vulnerability scanner for spring4shell

USAGE:
    spring4shell-scanner [OPTIONS] --targets <TARGETS>

OPTIONS:
    -h, --help                     Print help information
    -i, --targets <TARGETS>        Target file with urls to check, each url in a new line.
    -n, --num-tasks <NUM_TASKS>    Number of requests run concurrently [default: 10]
    -r, --rust-log <RUST_LOG>      Pass RUST_LOG (from env_logger crate) via cli. Supported:
                                   error, warn, info, debug and trace [default: error]
    -t, --timeout <TIMEOUT>        Maximal number of seconds till request timeout [default: 15]
    -V, --version                  Print version information

```

Note: The rust-log argument can also filter per module as described in:

- <https://docs.rs/env_logger/0.9.0/env_logger/>


## Build

```sh
$ cargo build --release
```


