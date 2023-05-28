use std::io::{self, Write};
use reqwest::blocking::Client;
use serde_json::Value;

fn main() {
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input == "exit" {
            break;
        }

        let mut parts = input.splitn(2, ' ');
        let method = parts.next().unwrap();
        let url = parts.next().unwrap();

        let response = match method {
            "GET" => send_get_request(url),
            "POST" => {
                print!("Body: ");
                io::stdout().flush().unwrap();

                let mut body = String::new();
                io::stdin().read_line(&mut body).unwrap();

                send_post_request(url, &body)
            }
            _ => {
                println!("Invalid method: {}", method);
                continue;
            }
        };

        match response {
            Ok(json) => {
                let pretty_json = serde_json::to_string_pretty(&json).unwrap();
                println!("{}", pretty_json);
            }
            Err(err) => {
                println!("Error: {}", err);
            }
        }
    }
}

fn send_get_request(url: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let client = Client::new();
    let response = client.get(url).send()?;
    let json: Value = response.json()?;
    Ok(json)
}

fn send_post_request(url: &str, body: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let client = Client::new();
    let response = client.post(url).body(body.to_string()).send()?;
    let json: Value = response.json()?;
    Ok(json)
}
