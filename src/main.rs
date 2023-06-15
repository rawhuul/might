use std::collections::HashMap;
use std::io::Write;
use std::num::NonZeroUsize;
use std::time::{Duration, Instant};

use ansi_term::{enable_ansi_support, Colour};
use argh::FromArgs;
use json_to_table::json_to_table;
use lru::LruCache;
use reqwest::blocking::Response;
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderName, HeaderValue},
    StatusCode,
};
use rustyline::{config::Configurer, error::ReadlineError, DefaultEditor};
use serde_json::{from_str, Value};
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
    printer: Printer,
    response_timeout: Duration,
    headers: HashMap<String, String>,
}

impl Session {
    fn new(json: bool, response_timeout: Option<u64>, cache_size: Option<u64>) -> Self {
        Session {
            cache: Cache::new(
                cache_size.unwrap_or(100).try_into().unwrap(),
                Duration::from_secs(5),
            ),
            history: HashMap::new(),
            printer: Printer {
                json,
                ..Default::default()
            },
            response_timeout: Duration::from_secs(response_timeout.unwrap_or(30)),
            headers: HashMap::new(),
        }
    }

    fn show_headers(&self) {
        if self.headers.len() == 0 {
            println!("[INFO]: No HEADERS :(");
            return;
        }

        println!("Session Headers:\n");
        for (header_name, header_content) in &self.headers {
            println!("{header_name}: {header_content}");
        }
    }

    fn show_history(&self) {
        if self.history.len() == 0 {
            println!("[INFO]: No History :(");
            return;
        }

        println!("Session History:\n");
        for (request, response) in &self.history {
            let pretty_request = request.replace(" ", " | ");
            let pretty_json = serde_json::to_string_pretty(&response).unwrap();
            println!("Request: {pretty_request}\nResponse: {pretty_json}\n");
        }
    }

    fn set_header(&mut self, name: &str) {
        if name.contains(|c: char| !c.is_alphanumeric()) {
            println!("[ERROR]: Invalid header name! Only alphanumeric characters are allowed.");
            return;
        }

        print!("Content: ");
        std::io::stdout().flush().unwrap();
        let mut body = String::new();
        std::io::stdin().read_line(&mut body).unwrap();

        let body = body.trim();

        match from_str::<Value>(body) {
            Ok(_) => {
                self.headers.insert(name.to_string(), body.to_string());
                println!("[INFO]: Header {} set successfully!", name);
            }
            Err(e) => {
                println!("[ERROR]: Invalid JSON format: {e}");
            }
        }
    }

    fn get_header(&self, name: &str) -> Result<HeaderMap, String> {
        let header = match self.headers.get(name) {
            Some(x) => x,
            None => return Err(format!("Header {name} doesn't exists.")),
        };

        let header_json: Value = match serde_json::from_str(header) {
            Ok(x) => x,
            Err(_) => return Err(format!("While converting to json.")),
        };

        let mut headers = HeaderMap::new();

        if let Some(json_object) = header_json.as_object() {
            for (k, v) in json_object {
                let header_name = match HeaderName::from_bytes(k.as_bytes()) {
                    Ok(x) => x,
                    Err(_) => return Err(format!("Reading header {k}")),
                };

                let header_value = match v.as_str() {
                    Some(x) => x,
                    None => return Err(format!("Empty field for {k}")),
                };
                let header_value = match HeaderValue::from_str(header_value) {
                    Ok(x) => x,
                    Err(_) => return Err(format!("Invalid Header Value for {k}")),
                };

                headers.insert(header_name, header_value);
            }
        }

        Ok(headers)
    }
}

struct Printer {
    time: bool,
    size: bool,
    status: bool,
    version: bool,
    header: bool,
    json: bool,
}

impl Default for Printer {
    fn default() -> Self {
        Printer {
            time: true,
            size: true,
            status: true,
            version: false,
            header: false,
            json: false,
        }
    }
}

impl Printer {
    fn metadata(&self, response: &Response) {
        if self.status {
            let status = response.status();
            let p = match status {
                StatusCode::OK => "Success!",
                StatusCode::NOT_FOUND => "Resource Not Found!",
                StatusCode::UNAUTHORIZED => "Unauthorized! Please provide credentials.",
                StatusCode::INTERNAL_SERVER_ERROR => "Internal Server Error! Retry request...",
                _ => "",
            };

            let s = status.to_string();

            let status = match status.as_u16() {
                200..=299 => Colour::Green.paint(s),
                300..=399 => Colour::Cyan.paint(s),
                400..=499 => Colour::Yellow.paint(s),
                500..=599 => Colour::Red.paint(s),
                _ => Colour::White.paint(s),
            };

            println!("{} {status} ({p})", Colour::White.bold().paint("Status:"));
        }

        if self.size {
            let size = match response.content_length() {
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
            };

            println!("{} {size}", Colour::White.bold().paint("Response Size:"));
        }

        if self.header {
            println!("{}", Colour::White.bold().paint("Response Header:"));

            for (name, value) in response.headers() {
                println!("{}: {:?}", name, value);
            }
        }

        if self.version {
            println!(
                "{} {:?}",
                Colour::White.bold().paint("Version:"),
                response.version()
            );
        }
    }

    fn time(&self, time: Duration) {
        if self.time {
            let secs = time.as_secs();
            let millis = time.subsec_millis();

            let time = if secs >= 60 {
                let mins = secs / 60;
                format!("{mins} min {}.{millis:03} s", secs % 60)
            } else if secs > 0 {
                format!("{secs}.{millis:03} s")
            } else {
                format!("{millis} ms")
            };

            println!("{} {time}", Colour::White.bold().paint("Response Time:"));
        }
    }

    fn response(&self, json: &Value) {
        if !self.json {
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
        "GET" => send_request("GET", header, url, None, session),
        "POST" | "PUT" | "PATCH" => {
            print!("Body: ");
            std::io::stdout().flush().unwrap();
            let mut body = String::new();
            std::io::stdin().read_line(&mut body).unwrap();
            send_request(method, header, url, Some(body), session);
        }

        "DELETE" => send_request("DELETE", header, url, None, session),
        _ => {
            println!("[ERROR]: Invalid method: {}", method);
        }
    }
}

fn send_request(
    method: &str,
    header: &str,
    url: &str,
    body: Option<String>,
    session: &mut Session,
) {
    session.cache.remove_expired_entries();

    let cache_key = format!("{} {}", method, url);

    if let Some(cached_response) = session.cache.get(&cache_key) {
        println!("[INFO] Using cached response");
        session.printer.response(&cached_response);
        return;
    }

    let client = Client::builder()
        .timeout(session.response_timeout)
        .build()
        .unwrap();

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
    let request = request.body(body.unwrap_or_else(String::new));

    let start_time = Instant::now();

    let response = request.send();

    match response {
        Ok(response) => {
            session.printer.metadata(&response);
            session
                .printer
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

            let json: Value = match response.json() {
                Ok(x) => x,
                Err(e) => {
                    println!("{e}");
                    return;
                }
            };

            session.history.insert(cache_key.clone(), json.clone());
            session.cache.put(cache_key.clone(), json.clone());
            session.printer.response(&json);
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
    cache_size: Option<u64>,
    #[argh(switch, short = 'j', description = "outputs in JSON (default: false)")]
    json: bool,
}

fn main() {
    let args: Args = argh::from_env();

    let mut session = Session::new(args.json, args.response_timeout, args.cache_size);

    repl(&mut session);
}
