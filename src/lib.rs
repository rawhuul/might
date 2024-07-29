use minreq::{Method, Request};
use rayon::prelude::*;
use std::collections::HashMap;

mod error {
    #[derive(Debug, Clone)]
    pub enum Error {
        InvalidMethod(String),
        InvalidSection(String),
        FailedToParseStatusCode(String),
        HeaderExpectsKV,
        PayloadExpectsKV,
        InvalidKeyInAssertions(String),
    }

    impl std::error::Error for Error {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            None
        }

        fn description(&self) -> &str {
            "description() is deprecated; use Display"
        }

        fn cause(&self) -> Option<&dyn std::error::Error> {
            self.source()
        }
    }

    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Error::InvalidMethod(x) => write!(f, "Method: {x} is invalid"),
                Error::InvalidSection(x) => write!(f, "Found {x} invalid section"),
                Error::FailedToParseStatusCode(x) => write!(f, "Failed to parse status code: {x}"),
                Error::HeaderExpectsKV => write!(f, "Header section expects key-value pair"),
                Error::PayloadExpectsKV => write!(f, "Payload section expects key-value pair"),
                Error::InvalidKeyInAssertions(x) => {
                    write!(f, "Found invalid key {x} in assertions section")
                }
            }
        }
    }
}

use error::Error;

mod result {
    pub enum TestCaseResult {
        Success { test: String },
        Fail { test: String, remarks: Remarks },
    }

    impl std::fmt::Display for TestCaseResult {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                TestCaseResult::Success { test } => {
                    write!(f, "✅ Successfully passed test: \"{test}\"")
                }
                TestCaseResult::Fail { test, remarks } => {
                    write!(f, "❌ Failed test: \"{test}\", remarks: {remarks}")
                }
            }
        }
    }

    pub enum Remarks {
        StatusCodeFail { expected: u16, recived: u16 },
        FailedRequest { error: minreq::Error },
    }

    impl std::fmt::Display for Remarks {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Remarks::StatusCodeFail { expected, recived } => {
                    write!(f, "expected status code: {expected}, got: {recived}")
                }
                Remarks::FailedRequest { error } => write!(f, "failed due to error: {error}"),
            }
        }
    }
}

use result::{Remarks, TestCaseResult};

pub struct TestCases(Vec<TestCase>);

impl TestCases {
    pub fn spawn(&self) -> Vec<TestCaseResult> {
        let Self(testcases) = self;

        testcases
            .par_iter()
            .map(|testcase| {
                let TestCase {
                    name,
                    description: _,
                    author: _,
                    method,
                    url,
                    status_code,
                    headers,
                    payload: _,
                    assertions: _,
                } = testcase;

                let Headers(headers) = headers;

                let request = Request::new(method.clone(), url).with_headers(headers);

                let response = request.send();

                match response {
                    Ok(res) => {
                        let status_resp = res.status_code == status_code.0 as i32;

                        if status_resp == false {
                            TestCaseResult::Fail {
                                test: name.to_string(),
                                remarks: Remarks::StatusCodeFail {
                                    expected: status_code.0,
                                    recived: res.status_code as u16,
                                },
                            }
                        } else {
                            TestCaseResult::Success {
                                test: name.to_string(),
                            }
                        }
                    }
                    Err(error) => TestCaseResult::Fail {
                        test: name.to_string(),
                        remarks: Remarks::FailedRequest { error },
                    },
                }
            })
            .collect()
    }
}

pub struct Parser;

impl Parser {
    pub fn parse(input: &str) -> Result<TestCases, Error> {
        let input = filter_out_comments(input);

        let testcases: Result<Vec<_>, Error> = input
            .split("---")
            .par_bridge()
            .map(|s| {
                let trimmed = s.trim();
                TestCase::parse(trimmed)
            })
            .collect();

        let testcases = testcases?;

        Ok(TestCases(testcases))
    }
}

fn filter_out_comments(input: &str) -> String {
    input
        .par_lines()
        .filter(|line| !line.trim().starts_with('#'))
        .collect::<Vec<&str>>()
        .join("\n")
}

#[derive(Debug)]
struct TestCase {
    name: String,
    description: String,
    author: Option<String>,
    method: Method,
    url: String,
    status_code: StatusCode,
    headers: Headers,
    payload: Payload,
    assertions: Assertions,
}

#[derive(Debug)]
struct StatusCode(u16);

impl TryFrom<&str> for StatusCode {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let status_code = value
            .parse::<u16>()
            .map_err(|_| Error::FailedToParseStatusCode(value.into()))?;

        Ok(Self(status_code))
    }
}

impl TestCase {
    fn parse(input: &str) -> Result<Self, Error> {
        let mut res = TestCase {
            name: String::new(),
            author: None,
            description: String::new(),
            method: Method::Custom("".to_owned()),
            url: String::new(),
            status_code: StatusCode(0),
            headers: Headers::new(),
            payload: Payload::new(),
            assertions: Default::default(),
        };

        let mut skip = 0usize;

        for line in input.lines() {
            if skip != 0 {
                skip -= 1;
            } else {
                let idx = line.find(':');

                if let Some(idx) = idx {
                    let (section, value) = (&line[..idx], &line[idx + 1..]);

                    match (section.to_lowercase().as_str(), value.trim()) {
                        ("testcase", value) => res.name = value.into(),
                        ("description", value) => res.description = value.into(),
                        ("author", value) => res.author = Some(value.into()),
                        ("url", value) => res.url = value.into(),
                        ("statuscode", value) => res.status_code = StatusCode::try_from(value)?,
                        ("method", value) => res.method = parse_method(value)?,
                        ("headers", _) => {
                            let headers = Headers::parse(input)?;
                            skip = headers.len();
                            res.headers = headers;
                        }
                        ("payload", _) => {
                            let payload = Payload::parse(input)?;
                            skip = payload.len();
                            res.payload = payload;
                        }
                        ("assertions", _) => {
                            let assertions = Assertions::parse(input)?;
                            skip = assertions.len();
                            res.assertions = assertions;
                        }
                        (x, _) => return Err(Error::InvalidSection(x.into())),
                    }
                } else {
                    continue;
                }
            }
        }

        Ok(res)
    }
}

fn parse_method(input: &str) -> Result<Method, Error> {
    match input.trim().to_uppercase().as_str() {
        "GET" => Ok(Method::Get),
        "POST" => Ok(Method::Post),
        "PUT" => Ok(Method::Put),
        "PATCH" => Ok(Method::Patch),
        "DELETE" => Ok(Method::Delete),
        "HEAD" => Ok(Method::Head),
        "TRACE" => Ok(Method::Trace),
        "OPTIONS" => Ok(Method::Options),
        "CONNECT" => Ok(Method::Connect),
        x => Err(Error::InvalidMethod(x.into())),
    }
}

#[derive(Debug)]
struct Headers(HashMap<String, String>);

impl Headers {
    fn new() -> Self {
        Self(HashMap::new())
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn parse(input: &str) -> Result<Self, Error> {
        let mut headers: HashMap<String, String> = HashMap::new();
        let mut in_header = false;

        for line in input.lines() {
            if line.to_lowercase().starts_with("headers") {
                in_header = true;
                continue;
            }

            if in_header {
                if line.starts_with("  ") || line.starts_with("\t") {
                    let idx = line.find(':');

                    match idx {
                        Some(i) => {
                            let k = line[..i].trim().to_string();
                            let v = line[i + 1..].trim().to_string();
                            headers.insert(k, v);
                        }
                        None => return Err(Error::HeaderExpectsKV),
                    }
                } else {
                    break;
                }
            }
        }

        Ok(Self(headers))
    }
}

#[derive(Debug)]
struct Payload(HashMap<String, String>);

impl Payload {
    fn new() -> Self {
        Self(HashMap::new())
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn parse(input: &str) -> Result<Self, Error> {
        let mut payloads: HashMap<String, String> = HashMap::new();
        let mut in_payload = false;

        for line in input.lines() {
            if line.to_lowercase().starts_with("payload") {
                in_payload = true;
                continue;
            }

            if in_payload {
                if line.starts_with("  ") || line.starts_with("\t") {
                    let idx = line.find(':');

                    match idx {
                        Some(i) => {
                            let k = line[..i].trim().to_string();
                            let v = line[i + 1..].trim().to_string();
                            payloads.insert(k, v);
                        }
                        None => return Err(Error::PayloadExpectsKV),
                    }
                } else {
                    break;
                }
            }
        }

        Ok(Self(payloads))
    }
}

#[derive(Debug, Default)]
struct Assertions {
    json_path_exists: Vec<Expr>,
    json_path_value: Vec<Expr>,
    header_exists: String,
    header_value: Vec<Expr>,
}

impl Assertions {
    fn new() -> Self {
        Self {
            json_path_exists: Vec::new(),
            json_path_value: Vec::new(),
            header_exists: String::new(),
            header_value: Vec::new(),
        }
    }

    fn len(&self) -> usize {
        self.json_path_exists.len() + self.json_path_value.len() + self.header_value.len() + 1
    }

    fn parse(input: &str) -> Result<Self, Error> {
        let mut assertions = Self::new();
        let mut in_assertions = false;

        for line in input.lines() {
            if line.to_lowercase().starts_with("assertions") {
                in_assertions = true;
                continue;
            }

            if in_assertions {
                if line.starts_with("  ") || line.starts_with("\t") {
                    let idx = line.find(':');

                    if let Some(idx) = idx {
                        let (key, value) = (&line[..idx], &line[idx + 1..]);

                        match (key.trim().to_lowercase().as_str(), value.trim()) {
                            ("jsonpathexists", v) => assertions.json_path_exists.push(v.into()),
                            ("jsonpathvalue", v) => assertions.json_path_value.push(v.into()),
                            ("headerexists", v) => assertions.header_exists = v.into(),
                            ("headervalue", v) => assertions.header_value.push(v.into()),
                            (x, _) => return Err(Error::InvalidKeyInAssertions(x.into())),
                        }
                    }
                } else {
                    break;
                }
            }
        }

        Ok(assertions)
    }
}

#[derive(Debug, Default)]
struct Expr(String);

impl Expr {
    fn new() -> Self {
        Self(String::new())
    }
}

impl<T> From<T> for Expr
where
    T: Into<String>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}
