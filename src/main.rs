use std::collections::HashMap;
use std::io::Write;
use std::time::Instant;

use ansi_term::enable_ansi_support;
use ansi_term::Colour;
use argh::FromArgs;
use json_to_table::json_to_table;
use reqwest::blocking::Client;
use reqwest::StatusCode;
use rustyline::{config::Configurer, error::ReadlineError, DefaultEditor};
use serde_json::{Value};
use tabled::settings::{style::RawStyle, Color, Style};

#[derive(Default)]
struct SessionHistory {
    history: HashMap<String, Value>,
}

fn repl(session: &mut SessionHistory, pretty_print: bool) {
    let mut rl = DefaultEditor::new().unwrap();

    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    if enable_ansi_support().is_err() {
        println!("Your system doesn't support ansi_colors.");
    }

    rl.set_color_mode(rustyline::ColorMode::Enabled);

    let prompt = format!(">>> ");

    loop {
        let readline = rl.readline(&prompt);
        match readline {
            Ok(input) => {
                let input = input.trim();

                if input.is_empty() {
                    continue;
                };

                if input == "exit" || input == "EXIT" || input == "Exit" {
                    break;
                }

                _ = rl.add_history_entry(input);

                match input {
                    "history" | "History" | "HISTORY" => show_history(session),
                    _ => {
                        process_input(input, session, pretty_print);
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

fn process_input(input: &str, session: &mut SessionHistory, pretty_print: bool) {
    let parts: Vec<&str> = input.split(' ').collect();

    if parts.len() != 2 {
        println!("Expected 2 arguments, found {}!", parts.len());
        return;
    }

    let method = parts[0];
    let url = parts[1];

    match method {
        "GET" => send_request("GET", url, None, session, pretty_print),
        "POST" | "PUT" | "PATCH" => {
            print!("Body: ");
            std::io::stdout().flush().unwrap();
            let mut body = String::new();
            std::io::stdin().read_line(&mut body).unwrap();
            send_request(method, url, Some(body), session, pretty_print);
        }

        "DELETE" => send_request("DELETE", url, None, session, pretty_print),
        _ => {
            println!("Invalid method: {}", method);
        }
    }
}

fn send_request(
    method: &str,
    url: &str,
    body: Option<String>,
    session: &mut SessionHistory,
    pretty_print: bool,
) {
    let client = Client::new();
    let start_time = Instant::now();

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
            let end_time = Instant::now();
            let duration = end_time.duration_since(start_time);

            println!(
                "{} {}",
                Colour::White.bold().paint("Response Time:"),
                format_duration(duration)
            );
            println!(
                "{} {}",
                Colour::White.bold().paint("Response Size:"),
                format_response_size(response.content_length())
            );
            println!("{}", handle_status_code(response.status()));

            let json: Value;

            match response.json() {
                Ok(x) => json = x,
                Err(e) => {
                    println!("{e}");
                    return;
                }
            }

            session
                .history
                .insert(format!("{} {}", method, url), json.clone());

            pprint(json, pretty_print);
        }
        Err(err) => {
            println!("Error: {}", err);
        }
    }
}

fn handle_status_code(status: StatusCode) -> String {
    let p = match status {
        StatusCode::OK => {
            format!("Success!")
        }
        StatusCode::NOT_FOUND => {
            format!("Resource Not Found!")
        }
        StatusCode::UNAUTHORIZED => {
            format!("Unauthorized! Please provide credentials.")
        }
        StatusCode::INTERNAL_SERVER_ERROR => {
            format!("Internal Server Error! Retrying request...")
        }
        _ => format!(""),
    };

    let s = format!("{}", status);

    let status = match status.as_u16() {
        200..=299 => Colour::Green.paint(s),
        300..=399 => Colour::Cyan.paint(s),
        400..=499 => Colour::Yellow.paint(s),
        500..=599 => Colour::Red.paint(s),
        _ => Colour::White.paint(s),
    };

    format!(
        "{} {} ({})",
        Colour::White.bold().paint("Status:"),
        status,
        p
    )
}


fn format_duration(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();

    if secs >= 60 {
        let mins = secs / 60;
        format!("{} min {}.{:03} s", mins, secs % 60, millis)
    } else if secs > 0 {
        format!("{}.{:03} s", secs, millis)
    } else {
        format!("{} ms", millis)
    }
}

fn format_response_size(size: Option<u64>) -> String {
    match size {
        Some(size) => {
            if size >= 1 << 30 {
                format!("{:.2} GB", size as f64 / (1 << 30) as f64)
            } else if size >= 1 << 20 {
                format!("{:.2} MB", size as f64 / (1 << 20) as f64)
            } else if size >= 1 << 10 {
                format!("{:.2} KB", size as f64 / (1 << 10) as f64)
            } else {
                format!("{} bytes", size)
            }
        }
        None => "Unknown".to_string(),
    }
}

fn pprint(json: Value, table: bool) {
    if table {
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

        println!("{}", json_to_table(&json).with(style));
    } else {
        match serde_json::to_string_pretty(&json) {
            Ok(result) => println!("{result}"),
            Err(e) => print!("{e}"),
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

#[derive(FromArgs)]
/// Simple command-line application that allows users to send HTTP requests and view the response, to test APIs.
struct Args {
    #[argh(switch, short = 'j', description = "outputs in JSON")]
    json: bool,
}

fn main() {
    let args: Args = argh::from_env();
    let mut session = SessionHistory::default();
    repl(&mut session, !args.json);
}
