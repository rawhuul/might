use reqwest::blocking::Client;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use serde_json::{json, Value};
use std::io::{self, Write};

fn repl(input: String) {
    let input = input.trim();

    if input.is_empty() {
        return;
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
        "PUT" => {
            print!("Body: ");
            io::stdout().flush().unwrap();

            let mut body = String::new();
            io::stdin().read_line(&mut body).unwrap();

            send_put_request(url, &body)
        }
        "PATCH" => {
            print!("Body: ");
            io::stdout().flush().unwrap();

            let mut body = String::new();
            io::stdin().read_line(&mut body).unwrap();

            send_patch_request(url, &body)
        }
        "DELETE" => send_delete_request(url),
        _ => {
            println!("Invalid method: {}", method);
            return;
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

fn main() {
    let mut rl = DefaultEditor::new().expect("Error Occured");
    loop {
        let readline = rl.readline(">>> ");
        match readline {
            Ok(input) => repl(input),

            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }

            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}

fn send_get_request(url: &str) -> std::result::Result<Value, Box<dyn std::error::Error>> {
    let client = Client::new();
    let response = client.get(url).send()?;
    let json: Value = response.json()?;
    Ok(json)
}

fn send_post_request(
    url: &str,
    body: &str,
) -> std::result::Result<Value, Box<dyn std::error::Error>> {
    let client = Client::new();
    let response = client.post(url).body(body.to_string()).send()?;
    let json: Value = response.json()?;
    Ok(json)
}

fn send_put_request(
    url: &str,
    body: &str,
) -> std::result::Result<Value, Box<dyn std::error::Error>> {
    let client = Client::new();
    let response = client.put(url).body(body.to_string()).send()?;
    let json: Value = response.json()?;
    Ok(json)
}

fn send_patch_request(
    url: &str,
    body: &str,
) -> std::result::Result<Value, Box<dyn std::error::Error>> {
    let client = Client::new();
    let response = client.patch(url).body(body.to_string()).send()?;
    let json: Value = response.json()?;
    Ok(json)
}

fn send_delete_request(url: &str) -> std::result::Result<Value, Box<dyn std::error::Error>> {
    let client = Client::new();
    let response = client.delete(url).send()?;
    let json: Value = response.json()?;
    Ok(json)
}
