use futures::stream::{self, StreamExt};
//use async_std::io::prelude::*;
//use async_std::io;
//use surf::{Client, StatusCode, Response};
use reqwest::{Client, ClientBuilder, Response, StatusCode};
use tokio::io::{self, AsyncBufReadExt};
use tokio_stream::wrappers::LinesStream;

// Usage:
//   cat targets.lst| cargo run -- 20
//   cat targets.lst| RUST_LOG='spring4shell_scanner=Debug' cargo run -- 20

const USER_AGENT: &str = "spring4shell-scanner";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

#[derive(Debug)]
enum Finding {
    Get(String),
    Post(String),
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

async fn do_get_request(client: &Client, target: &str, vector: &str) -> Result<Response> {
    let res = client.get(format!("{}?{}", target, vector)).send().await?;

    Ok(res)
}

async fn check(client: &Client, target: &str) -> Result<Option<Finding>> {
    // curl -v -XPOST http://192.168.178.43:8080/helloworld/greeting \
    //     -d'class.module.classLoader.URLs%5B0%5D=0' |& grep 'HTTP/1.1 '
    //
    // src: https://twitter.com/RandoriAttack/status/1509298490106593283

    let test_vector_400 = "class.module.classLoader.URLs%5B0%5D=0";
    let test_vector_not_400 = "class.module.classXLoader.URLs%5B0%5D=0";

    // test GET request

    let res_400 = do_get_request(client, target, test_vector_400).await?;
    log::debug!("[{}] GET response code -> {}", target, res_400.status());

    if res_400.status() == StatusCode::BAD_REQUEST {
        // we got a 400 response lets check if it was a fp by sending a
        // invalid input vector
        let res_not_400 = do_get_request(client, target, test_vector_not_400).await?;

        log::debug!("[{}] GET response code -> {}", target, res_not_400.status());
        if res_not_400.status() != StatusCode::BAD_REQUEST {
            return Ok(Some(Finding::Get(target.into())));
        }
    }

    // test POST request

    let res_400 = do_post_request(client, target, test_vector_400).await?;
    log::debug!("[{}] POST response code -> {}", target, res_400.status());

    if res_400.status() == StatusCode::BAD_REQUEST {
        // we got a 400 response lets check if it was a fp by sending a
        // invalid input vector
        let res_not_400 = do_post_request(client, target, test_vector_not_400).await?;

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

//#[async_std::main]
#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // args parsing
    let args: Vec<String> = std::env::args().collect();
    let num_tasks: u16 = args.get(1).unwrap_or(&"10".into()).parse()?;

    let client = ClientBuilder::new()
            .user_agent(USER_AGENT)
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true)
            .build()?;

    let clients_stream = stream::iter(std::iter::repeat(1).map(|_| client.clone()));

    // read targets from stdin
    let reader = io::BufReader::new(io::stdin());
    let targets_stream = reader.lines();

    // just do it
    let findings = clients_stream
        .zip(LinesStream::new(targets_stream))
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
        .buffer_unordered(num_tasks as usize)
        .flat_map(stream::iter)
        .inspect(|finding| {
             match finding {
                Finding::Get(url) => log::info!("Found: GET {}", url),
                Finding::Post(url) => log::info!("Found: POST {}", url),
             };
        })
        .collect::<Vec<Finding>>()
        .await;

    println!("Final Result List:");
    for finding in &findings {
        match finding {
            Finding::Get(url) => println!("GET {}", url),
            Finding::Post(url) => println!("POST {}", url),
        };
    }

    Ok(())
}
