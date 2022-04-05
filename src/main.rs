use std::time::Duration;

use reqwest::{Client, ClientBuilder, Response, StatusCode};

use futures::stream::{self, StreamExt};

use tokio::{
    fs,
    io::{self, AsyncBufReadExt}
};
use tokio_stream::wrappers::LinesStream;

use clap::Parser;



const USER_AGENT: &str = "spring4shell-scanner";
const RUST_LOG: &str = "RUST_LOG";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

#[derive(Debug)]
enum Finding {
    Get(String),
    Post(String),
}

#[derive(Parser, Debug)]
#[clap(version, about)]
struct Args {
    #[clap(short, long,
           default_value_t = 10,
           help = "Number of requests run concurrently")]
    num_tasks: u32,

    #[clap(short, long,
           default_value_t = 15,
           help = "Maximal number of seconds till request timeout"
           )]
    timeout:   u32,

    #[clap(short, long,
           default_value = "error",
           help = "Pass RUST_LOG (from env_logger crate) via cli. \
           Supported:\nerror, warn, info, debug and trace")]
    rust_log:  String,

    #[clap(short = 'i', long, required = true,
           help = "Target file with urls to check, each url in a new line.")]
    targets: String,
}


async fn do_post_request(client: &Client, target: &str, vector: &str) -> Result<Response> {
    let res = client
        .post(target)
        .body(vector.to_owned())
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await?;

    Ok(res)
}

async fn do_get_request(
    client: &Client,
    target: &str,
    vector: &[(&str, &str)],
) -> Result<Response> {
    let res = client.get(target).query(vector).send().await?;

    Ok(res)
}

async fn check(client: &Client, target: &str) -> Result<Option<Finding>> {
    // curl -v -XPOST http://192.168.178.43:8080/helloworld/greeting \
    //     -d'class.module.classLoader.URLs%5B0%5D=0' |& grep 'HTTP/1.1 '
    //
    // src: https://twitter.com/RandoriAttack/status/1509298490106593283

    let test_400_post = "class.module.classLoader.URLs%5B0%5D=0";
    let test_not_400_post = "class.module.classXLoader.URLs%5B0%5D=0";
    let test_400_get = [("class.module.classLoader.URLs[0]", "0")];
    let test_not_400_get = [("class.module.classXLoader.URLs[0]", "0")];

    // test GET request

    let res_400 = do_get_request(client, target, &test_400_get).await?;
    log::debug!("[{}] GET response code -> {}", target, res_400.status());

    if res_400.status() == StatusCode::BAD_REQUEST {
        // we got a 400 response lets check if it was a fp by sending a
        // invalid input vector
        let res_not_400 = do_get_request(client, target, &test_not_400_get).await?;

        log::debug!("[{}] GET response code -> {}", target, res_not_400.status());
        if res_not_400.status() != StatusCode::BAD_REQUEST {
            return Ok(Some(Finding::Get(target.into())));
        }
    }

    // test POST request

    let res_400 = do_post_request(client, target, test_400_post).await?;
    log::debug!("[{}] POST response code -> {}", target, res_400.status());

    if res_400.status() == StatusCode::BAD_REQUEST {
        // we got a 400 response lets check if it was a fp by sending a
        // invalid input vector
        let res_not_400 = do_post_request(client, target, test_not_400_post).await?;

        log::debug!(
            "[{}] POST response code -> {}",
            target,
            res_not_400.status()
        );
        if res_not_400.status() != StatusCode::BAD_REQUEST {
            return Ok(Some(Finding::Post(target.into())));
        }
    }

    Ok(None)
}


#[tokio::main]
async fn main() -> Result<()> {

    let args = Args::parse();
    if std::env::var(RUST_LOG).is_err() {
        std::env::set_var(RUST_LOG, args.rust_log);
    }
    env_logger::init();

    let client = ClientBuilder::new()
        .timeout(Duration::from_secs(args.timeout as u64))
        .user_agent(USER_AGENT)
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .build()?;

    let clients_stream = stream::iter(std::iter::repeat(1).map(|_| client.clone()));

    let reader = io::BufReader::new(fs::File::open(args.targets).await?);
    let targets_stream = LinesStream::new(reader.lines());


    // just do it
    let findings = clients_stream
        .zip(targets_stream)
        .map(|(client, target)| async move {
            match check(&client, target.as_ref().unwrap()).await {
                // filter out all errors
                Ok(finding) => finding,
                Err(err) => {
                    log::error!("{}: {}", target.as_ref().unwrap(), err);
                    None
                }
            }
        })
        .buffer_unordered(args.num_tasks as usize)
        .inspect(|_| print!(".")) // progress indicator
        .flat_map(stream::iter)   // filer out None
        .inspect(|finding| {
            match finding {
                Finding::Get(url) => log::info!("Found: GET {}", url),
                Finding::Post(url) => log::info!("Found: POST {}", url),
            };
        })
        .collect::<Vec<Finding>>()
        .await;

    println!("\n[+] Final Result List:");
    for finding in &findings {
        match finding {
            Finding::Get(url) => println!("GET {}", url),
            Finding::Post(url) => println!("POST {}", url),
        };
    }

    Ok(())
}
