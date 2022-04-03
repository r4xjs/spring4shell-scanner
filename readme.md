# spring4shell\_scanner

Network scanner based on Tokio async runtime for detecting spring4shell
vulnerability. The scanner currently supports GET and POST request.
The scanner will read target endpoints from stdin and takes an optional number
of tasks via cli parameter (default is 10).

The detection is based on the method posted by RandoriAttack:

- https://twitter.com/RandoriAttack/status/1509298490106593283


## Build

```sh
cargo build --release
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
