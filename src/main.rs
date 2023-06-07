use std::collections::HashMap;
use std::io::Write;
use std::num::NonZeroUsize;
use std::time::{Duration, Instant};

use ansi_term::{enable_ansi_support, Colour};
use argh::FromArgs;
use json_to_table::json_to_table;
use lru::LruCache;
use reqwest::{blocking::Client, StatusCode};
use rustyline::{config::Configurer, error::ReadlineError, DefaultEditor};
use serde_json::Value;
use tabled::settings::{style::RawStyle, Color, Style};

struct Cache {
    cache: LruCache<String, (Value, Instant)>,
    max_size: usize,
    max_age: Duration,
}

impl Cache {
    fn new(max_size: usize, max_age: Duration) -> Self {
        Cache {
            cache: LruCache::new(NonZeroUsize::new(max_size).unwrap()),
            max_size,
            max_age,
        }
    }

    fn get(&mut self, key: &str) -> Option<Value> {
        if let Some((value, timestamp)) = self.cache.get_mut(key) {
            if timestamp.elapsed() <= self.max_age {
                return Some(value.clone());
            }
        }
        None
    }

    fn put(&mut self, key: String, value: Value) {
        let timestamp = Instant::now();
        self.cache.put(key, (value, timestamp));

        if self.cache.len() > self.max_size {
            self.cache.pop_lru();
        }
    }

    fn remove_expired_entries(&mut self) {
        let now = Instant::now();

        let expired_keys: Vec<String> = self
            .cache
            .iter()
            .filter(|(_, (_, timestamp))| now.duration_since(*timestamp) > self.max_age)
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            self.cache.pop(&key);
        }
    }
}

struct Session {
    cache: Cache,
    history: HashMap<String, Value>,
    pretty_print: bool,
    response_timeout: Duration,
}

impl Session {
    fn new(pretty_print: bool, response_timeout: Option<u64>, cache_size: Option<u64>) -> Self {
        Session {
            cache: Cache::new(cache_size.unwrap_or(100).try_into().unwrap(), Duration::from_secs(5)),
            history: HashMap::new(),
            pretty_print,
            response_timeout: Duration::from_secs(response_timeout.unwrap_or(30))
        }
    }

    fn show_history(&self) {
        if self.history.len() == 0 {
            println!("No History :(");
            return;
        }

        println!("Session History:\n");
        for (request, response) in &self.history {
            let pretty_request = request.replace(" ", " | ");
            let pretty_json = serde_json::to_string_pretty(&response).unwrap();
            println!("Request: {pretty_request}\nResponse: {pretty_json}\n");
        }
    }
}

fn repl(session: &mut Session) {
    let mut rl = DefaultEditor::new().unwrap();

    if rl.load_history("history.txt").is_err() {
        println!("[INFO]: No previous history.");
    }

    if enable_ansi_support().is_err() {
        println!("[ERROR]: Your system doesn't support ansi_colors.");
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
                    println!("[INFO]: Goodbye!");
                    break;
                }

                _ = rl.add_history_entry(input);

                match input {
                    "history" | "History" | "HISTORY" => session.show_history(),
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

    if parts.len() != 2 {
        println!("[ERROR]: Expected 2 arguments, found {}!", parts.len());
        return;
    }

    let method = parts[0];
    let url = parts[1];

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
            println!("[ERROR]: Invalid method: {}", method);
        }
    }
}

fn send_request(method: &str, url: &str, body: Option<String>, session: &mut Session) {
    session.cache.remove_expired_entries();

    let cache_key = format!("{} {}", method, url);

    if let Some(cached_response) = session.cache.get(&cache_key) {
        println!("[INFO] Using cached response");
        pprint(&cached_response, session.pretty_print);
        return;
    }

    let client = Client::builder()
        .timeout(session.response_timeout)
        .build()
        .unwrap();

    let start_time = Instant::now();

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

            let content_type = response.headers().get(reqwest::header::CONTENT_TYPE).and_then(
                |value| value.to_str().ok()
                ).unwrap_or("");

            if content_type.contains("text/html") {
                let html = response.text().unwrap();
                println!("[INFO]: Response is in HTML format.\n");
                println!("{}", html);
                return;
            }

            let json: Value;

            match response.json() {
                Ok(x) => json = x,
                Err(e) => {
                    println!("{e}");
                    return;
                }
            }

            session.history.insert(cache_key.clone(), json.clone());
            session.cache.put(cache_key.clone(), json.clone());

            pprint(&json, session.pretty_print);
        }
        Err(err) => {
            let e = Colour::Red.dimmed().paint("[ERROR]");

            if err.is_timeout() {
                println!(
                    "{e}: Response time exceeded the specified timeout of {} seconds.",
                    session.response_timeout.as_secs()
                );
            } else if err.is_decode() {
                println!("{e}: Failed to decode response.");
            } else {
                println!("{e}: {err}");
            }
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
            format!("Internal Server Error! Retry request...")
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

fn pprint(json: &Value, table: bool) {
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
            Err(e) => print!("[ERROR]: {e}"),
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
    #[argh(
        option,
        short = 'c',
        description = "cache size (default: 100)"
    )]
    cache_size: Option<u64>,
    #[argh(switch, short = 'j', description = "outputs in JSON (default: false)")]
    json: bool,
}

fn main() {
    let args: Args = argh::from_env();

    let mut session = Session::new(!args.json, args.response_timeout, args.cache_size);

    repl(&mut session);
}
