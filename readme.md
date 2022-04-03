# spring4shell\_scanner

Network scanner based on Tokio async runtime for detecting the spring4shell
vulnerability (CVE-2022-22965). Currently GET and POST request are checked.
The scanner will read target endpoints from stdin and takes the optional number
of tasks via cli parameter (default is 10).

The detection method is based on the curl command posted by RandoriAttack:

- https://twitter.com/RandoriAttack/status/1509298490106593283


## Build

```sh
$ cargo build --release

# or for static build

$ rustup target install x86_64-unknown-linux-musl
$ cargo build --release --target x86_64-unknown-linux-musl
```


## Run

```sh
$ cat targets.lst | head -1
https://f00.bar.to/path

$ cat targets.lst | target/release/spring4shell [num-tasks]
$ cat targets.lst | target/release/spring4shell
$ cat targets.lst | target/release/spring4shell 20

# enable debug login
$ cat targets.lst | RUST_LOG='spring4shell_scanner=Debug' target/release/spring4shell 20
```
