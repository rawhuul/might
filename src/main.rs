pub mod cache;
pub mod formatter;
pub mod session;

use ansi_term::{enable_ansi_support, Colour};
use argh::FromArgs;
use reqwest::{blocking::Client, header::HeaderMap};
use rustyline::{config::Configurer, error::ReadlineError, DefaultEditor};
use serde_json::Value;
use std::borrow::Cow;
use std::io::Write;
use std::time::Instant;

use session::Session;

fn repl(session: &mut Session) {
    let mut rl = DefaultEditor::new().unwrap();

    if rl.load_history("history.txt").is_err() {
        println!("[INFO]: No previous history.");
    }

    if enable_ansi_support().is_err() {
        println!("[ERROR]: Your system doesn't support ansi_colors.");
    }

    rl.set_color_mode(rustyline::ColorMode::Enabled);

    let prompt = ">>> ".to_string();

    loop {
        let readline = rl.readline(&prompt);
        match readline {
            Ok(input) => {
                let input = input.trim();

                if input.is_empty() {
                    continue;
                };

                if input.eq_ignore_ascii_case("exit") {
                    println!("[INFO]: Goodbye!");
                    break;
                }

                _ = rl.add_history_entry(input);

                match input {
                    "history" | "History" | "HISTORY" => session.show_history(),
                    "headers" | "Headers" | "HEADERS" => session.show_headers(),
                    _ => {
                        process_input(input, session);
                    }
                }
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
                println!("[INFO]: Goodbye!");
                break;
            }
            Err(err) => {
                println!("[ERROR]: {:?}", err);
                break;
            }
        }
    }
    _ = rl.save_history("history.txt");
}

fn process_input(input: &str, session: &mut Session) {
    let parts: Vec<&str> = input.split(' ').collect();

    if parts[0] == "HEADER" {
        if parts.len() == 2 {
            session.set_header(parts[1]);
            return;
        } else {
            println!("[ERROR]: Expected 2 arguments, found {}!", parts.len());
            return;
        }
    }

    if parts.len() != 3 {
        println!("[ERROR]: Expected 3 arguments, found {}!", parts.len());
        return;
    }

    let method = parts[0];
    let header = parts[1];
    let url = parts[2];

    match method {
        "GET" => send_request("GET", header, url, Cow::Borrowed(""), session),
        "POST" | "PUT" | "PATCH" => {
            print!("Body: ");
            std::io::stdout().flush().unwrap();
            let mut body = String::new();
            std::io::stdin().read_line(&mut body).unwrap();
            send_request(method, header, url, body.trim().into(), session);
        }

        "DELETE" => send_request("DELETE", header, url, Cow::Borrowed(""), session),
        _ => {
            println!("[ERROR]: Invalid method: {}", method);
        }
    }
}

fn send_request(method: &str, header: &str, url: &str, body: Cow<str>, session: &mut Session) {
    session.cache.remove_expired_entries();

    let cache_key = format!("{} {}", method, url);

    if let Some(cached_response) = session.cache.get(&cache_key) {
        println!("[INFO] Using cached response");
        session.formatter.response(cached_response);
        return;
    }

    let headers = if header != "{}" {
        match session.get_header(header) {
            Ok(x) => x,
            Err(e) => {
                println!("[ERROR]: {e}");
                return;
            }
        }
    } else {
        HeaderMap::new()
    };

    let client = Client::builder()
        .timeout(session.response_timeout)
        .build()
        .unwrap();

    let request = match method {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "PATCH" => client.patch(url),
        "DELETE" => client.delete(url),
        _ => {
            println!("[ERROR]: Invalid method: {}", method);
            return;
        }
    };

    let request = request.headers(headers);
    let request = request.body(body.to_string());

    let start_time = Instant::now();

    let response = request.send();

    match response {
        Ok(response) => {
            session.formatter.metadata(&response);
            session
                .formatter
                .time(Instant::now().duration_since(start_time));

            if response
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .unwrap_or("")
                .contains("text/html")
            {
                let html = response.text().unwrap();
                print!("[WARN]: Response is in HTML format.\nDo you want to print it? [y/n]: ");
                std::io::stdout().flush().unwrap();
                let mut body = String::new();
                std::io::stdin().read_line(&mut body).unwrap();

                if body.trim().eq_ignore_ascii_case("y") {
                    println!("{}", html);
                }

                return;
            }

            let json: Value = response.json().unwrap();
            session.history.insert(cache_key.clone(), json.clone());
            session.cache.put(cache_key.clone(), json.clone());
            session.formatter.response(&json);
        }
        Err(err) => {
            let e = Colour::Red.dimmed().paint("[ERROR]");
            let s = session.response_timeout.as_secs();

            if err.is_timeout() {
                println!("{e}: Response time exceeded the specified timeout of {s} seconds.");
            } else if err.is_decode() {
                println!("{e}: Failed to decode response.");
            } else {
                println!("{e}: {err}");
            }
        }
    }
}

#[derive(FromArgs)]
/// Simple command-line application that allows users to send HTTP requests and view the response, to test APIs.
struct Args {
    #[argh(
        option,
        short = 't',
        description = "response timeout in seconds (default: 30s)"
    )]
    response_timeout: Option<u64>,
    #[argh(option, short = 'c', description = "cache size (default: 100)")]
    cache_size: Option<usize>,
    #[argh(switch, short = 'j', description = "outputs in JSON (default: false)")]
    json: bool,
}

fn main() {
    let args: Args = argh::from_env();

    repl(&mut Session::new(
        args.json,
        args.response_timeout,
        args.cache_size,
    ));
}
