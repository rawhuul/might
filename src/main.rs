use std::collections::HashMap;
use std::io::Write;

use json_to_table::json_to_table;
use reqwest::blocking::Client;
use rustyline::{DefaultEditor, Result, error::ReadlineError};
use serde_json::{json, Value};
use tabled::settings::{style::RawStyle, Style, Color};

#[derive(Default)]
struct SessionHistory {
    history: HashMap<String, Value>,
}

fn repl(session: &mut SessionHistory) {
    let mut rl = DefaultEditor::new().unwrap();

    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    loop {
        let readline = rl.readline(">>> ");
        match readline {
            Ok(input) => {
                let input = input.trim();

                if input.is_empty() {
                    continue;
                }

                _ = rl.add_history_entry(input);

                match input {
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
    _ = rl.save_history("history.txt");
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
            session
                .history
                .insert(format!("{} {}", method, url), json.clone());

            // let pretty_json = serde_json::to_string_pretty(&json).unwrap();
            let mut json = json_to_table(&json);

            let mut style = RawStyle::from(Style::rounded());
            style
                .set_color_top(Color::FG_RED)
                .set_color_bottom(Color::FG_CYAN)
                .set_color_left(Color::FG_BLUE)
                .set_color_right(Color::FG_GREEN)
                .set_color_corner_top_left(Color::FG_BLUE)
                .set_color_corner_top_right(Color::FG_RED)
                .set_color_corner_bottom_left(Color::FG_CYAN)
                .set_color_corner_bottom_right(Color::FG_GREEN)
                .set_color_intersection_bottom(Color::FG_CYAN)
                .set_color_intersection_top(Color::FG_RED)
                .set_color_intersection_right(Color::FG_GREEN)
                .set_color_intersection_left(Color::FG_BLUE)
                .set_color_intersection(Color::FG_MAGENTA)
                .set_color_horizontal(Color::FG_MAGENTA)
                .set_color_vertical(Color::FG_MAGENTA);

            println!("{}", json.with(style));
        }
        Err(err) => {
            println!("Error: {}", err);
        }
    }
}

fn show_history(session: &SessionHistory) {
    if session.history.len() == 0 {
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
