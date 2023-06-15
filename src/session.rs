use crate::cache::Cache;
use crate::formatter::Formatter;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::{from_str, Value};
use std::collections::HashMap;
use std::io::Write;
use std::time::Duration;

pub struct Session {
    pub cache: Cache,
    pub history: HashMap<String, Value>,
    pub printer: Formatter,
    pub response_timeout: Duration,
    headers: HashMap<String, String>,
}

impl Session {
    pub fn new(json: bool, response_timeout: Option<u64>, cache_size: Option<u64>) -> Self {
        Session {
            cache: Cache::new(
                cache_size.unwrap_or(100).try_into().unwrap(),
                Duration::from_secs(5),
            ),
            history: HashMap::new(),
            printer: Formatter::new(json),
            response_timeout: Duration::from_secs(response_timeout.unwrap_or(30)),
            headers: HashMap::new(),
        }
    }

    pub fn show_headers(&self) {
        if self.headers.is_empty() {
            println!("[INFO]: No HEADERS :(");
            return;
        }

        println!("Session Headers:\n");
        for (header_name, header_content) in &self.headers {
            println!("{header_name}: {header_content}");
        }
    }

    pub fn show_history(&self) {
        if self.history.is_empty() {
            println!("[INFO]: No History :(");
            return;
        }

        println!("Session History:\n");
        for (request, response) in &self.history {
            let pretty_request = request.replace(' ', " | ");
            let pretty_json = serde_json::to_string_pretty(&response).unwrap();
            println!("Request: {pretty_request}\nResponse: {pretty_json}\n");
        }
    }

    pub fn set_header(&mut self, name: &str) {
        if !name.chars().all(char::is_alphanumeric) {
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

    pub fn get_header(&self, name: &str) -> Result<HeaderMap, String> {
        let header = match self.headers.get(name) {
            Some(x) => x,
            None => return Err(format!("Header {name} doesn't exists.")),
        };

        let header_json: Value = match serde_json::from_str(header) {
            Ok(x) => x,
            Err(_) => return Err("While converting to json.".to_string()),
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
