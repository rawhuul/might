use ansi_term::Colour;
use json_to_table::json_to_table;
use reqwest::{blocking::Response, StatusCode};
use serde_json::Value;
use std::time::Duration;
use tabled::settings::{style::RawStyle, Color, Style};

pub struct Formatter {
    time: bool,
    size: bool,
    status: bool,
    version: bool,
    header: bool,
    json: bool,
    style: RawStyle,
}

impl Formatter {
    pub fn new(json: bool) -> Self {
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

        Self {
            json,
            style,
            time: true,
            size: true,
            status: true,
            version: false,
            header: false,
        }
    }

    pub fn metadata(&self, response: &Response) {
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

    pub fn time(&self, time: Duration) {
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

    pub fn response(&self, json: &Value) {
        if !self.json {
            let style = self.style.clone();
            println!("{}", json_to_table(json).with(style));
        } else {
            match serde_json::to_string_pretty(&json) {
                Ok(result) => println!("{result}"),
                Err(e) => print!("[ERROR]: {e}"),
            }
        }
    }
}
