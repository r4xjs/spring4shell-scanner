# spring4shell-scanner

Network scanner based on Tokio async runtime for detecting the spring4shell
vulnerability (CVE-2022-22965). Currently GET and POST request are checked.
Vulernable endpoints will be shown during execution and a complete list
is also printed when finish.

The detection method is based on the curl command posted by RandoriAttack:

- https://twitter.com/RandoriAttack/status/1509298490106593283


## Usage

```
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

## Example

```
$ spring4shell-scanner --rust-log spring4shell_scanner=debug --targets /tmp/aaa.lst
[2022-04-05T22:36:30Z DEBUG spring4shell_scanner] [https://www.google.com] GET response code -> 200 OK
[2022-04-05T22:36:30Z DEBUG spring4shell_scanner] [https://www.google.com] POST response code -> 405 Method Not Allowed
[2022-04-05T22:36:30Z DEBUG spring4shell_scanner] [https://4chan.org] GET response code -> 200 OK
[2022-04-05T22:36:31Z DEBUG spring4shell_scanner] [https://github.com/r4xjs] GET response code -> 200 OK
[2022-04-05T22:36:31Z DEBUG spring4shell_scanner] [https://github.com/r4xjs] POST response code -> 403 Forbidden
[2022-04-05T22:36:31Z DEBUG spring4shell_scanner] [https://4chan.org] POST response code -> 200 OK
...
[+] Final Result List:
```


## Build

```
$ cargo build --release
```


