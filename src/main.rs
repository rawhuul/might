use std::collections::HashMap;
use std::io::Write;

use reqwest::blocking::Client;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use serde_json::{json, Value};

#[derive(Default)]
struct SessionHistory {
    history: HashMap<String, Value>,
}

fn repl(session: &mut SessionHistory) {
    let mut rl = DefaultEditor::new().unwrap();
    let mut history: Vec<String> = Vec::new();

    loop {
        let readline = rl.readline(">>> ");
        match readline {
            Ok(input) => {
                let input = input.trim();

                if input.is_empty() {
                    continue;
                }

                _ = rl.add_history_entry(input);
                history.push(input.to_string());

                match input {
                    "\x1b[A" => {
                        // Up arrow key
                        if let Some(prev_command) = history.get(history.len() - 2) {
                            println!(">>> {}", prev_command);
                            repl(session);
                        }
                    }
                    "\x1b[B" => {
                        // Down arrow key
                        if let Some(next_command) = history.get(history.len() - 1) {
                            println!(">>> {}", next_command);
                            repl(session);
                        }
                    }
                    "history" | "History" | "HISTORY" => show_history(session),

                    _ => {
                        process_input(input, session);
                    }
                }
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
                println!("Goodbye!");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}

fn process_input(input: &str, session: &mut SessionHistory) {
    let mut parts = input.splitn(2, ' ');
    let method = parts.next().unwrap();
    let url = parts.next().unwrap();

    match method {
        "GET" => send_request("GET", url, None, session),
        "POST" | "PUT" | "PATCH" => {
            print!("Body: ");
            std::io::stdout().flush().unwrap();
            let mut body = String::new();
            std::io::stdin().read_line(&mut body).unwrap();
            send_request(method, url, Some(body), session);
        }
        "DELETE" => send_request("DELETE", url, None, session),
        _ => {
            println!("Invalid method: {}", method);
        }
    }
}

fn send_request(method: &str, url: &str, body: Option<String>, session: &mut SessionHistory) {
    let client = Client::new();

    let request = match method {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "PATCH" => client.patch(url),
        "DELETE" => client.delete(url),
        _ => {
            println!("Invalid method: {}", method);
            return;
        }
    };

    let request = match body {
        Some(body) => request.body(body),
        None => request,
    };

    let response = request.send();

    match response {
        Ok(response) => {
            let json: Value = response.json().unwrap();
            let request_string = format!("{} {}", method, url);
            session.history.insert(request_string, json.clone());
            let pretty_json = serde_json::to_string_pretty(&json).unwrap();
            println!("{}", pretty_json);
        }
        Err(err) => {
            println!("Error: {}", err);
        }
    }
}

fn show_history(session: &SessionHistory) {
    if session.history.len() == 0{
        println!("No History :(");
        return;
    }

    println!("Session History:");
    for (request, response) in &session.history {
        let pretty_request = request.replace(" ", " | ");
        let pretty_json = serde_json::to_string_pretty(&response).unwrap();
        println!("Request: {pretty_request}\nResponse: {pretty_json}\n");
    }
}

fn main() {
    let mut session = SessionHistory::default();
    repl(&mut session);
}
