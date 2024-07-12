use std::{collections::HashMap, error::Error, ops::Not};

pub struct MIG(Vec<TestCase>);

impl MIG {
    pub fn parse(input: &str) -> Result<Self, Box<dyn Error>> {
        let input = filter_out_comments(input);

        let raw_testcases: Vec<&str> = input
            .split("---")
            .filter_map(|s| {
                let trimmed = s.trim();
                trimmed.is_empty().not().then(|| trimmed)
            })
            .collect();

        let mut testcases: Vec<TestCase> = vec![];

        for raw_testcase in raw_testcases {
            let testcase = TestCase::parse(raw_testcase)?;
            testcases.push(testcase);
        }

        println!("{testcases:#?}");

        Ok(Self(vec![]))
    }
}

fn filter_out_comments(input: &str) -> String {
    input
        .lines()
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
    status_code: u16,
    headers: Headers,
    payload: Payload,
    assertions: Assertions,
}

impl TestCase {
    fn parse(input: &str) -> Result<Self, Box<dyn Error>> {
        let mut res = TestCase {
            name: String::new(),
            author: None,
            description: String::new(),
            method: Method::NONE,
            url: String::new(),
            status_code: 0,
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
                        ("statuscode", value) => res.status_code = value.parse::<u16>()?,
                        ("method", value) => res.method = Method::parse(value)?,
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
                        (x, _) => return Err(format!("Unknow section \"{x}\"").into()),
                    }
                } else {
                    continue;
                }
            }
        }

        Ok(res)
    }
}

#[derive(Debug)]
enum Method {
    POST,
    PUT,
    PATCH,
    GET,
    DELETE,
    UPDATE,
    HEAD,
    TRACE,
    OPTIONS,
    CONNECT,
    NONE,
}

impl Method {
    fn parse(input: &str) -> Result<Self, Box<dyn Error>> {
        match input.trim().to_uppercase().as_str() {
            "GET" => Ok(Self::GET),
            "POST" => Ok(Self::POST),
            "PUT" => Ok(Self::PUT),
            "PATCH" => Ok(Self::PATCH),
            "DELETE" => Ok(Self::DELETE),
            "UPDATE" => Ok(Self::UPDATE),
            "HEAD" => Ok(Self::HEAD),
            "TRACE" => Ok(Self::TRACE),
            "OPTIONS" => Ok(Self::OPTIONS),
            "CONNECT" => Ok(Self::CONNECT),
            x => Err(format!("Method \"{x}\" is not supported").into()),
        }
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

    fn parse(input: &str) -> Result<Self, Box<dyn Error>> {
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
                        None => return Err(format!("Expected key-value pair").into()),
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

    fn parse(input: &str) -> Result<Self, Box<dyn Error>> {
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
                        None => return Err(format!("Expected key-value pair").into()),
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

    fn parse(input: &str) -> Result<Self, Box<dyn Error>> {
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
                            (x, _) => {
                                return Err(
                                    format!("Unknown key: \"{x}\" found in assert section").into()
                                )
                            }
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

impl From<&str> for Expr {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<&String> for Expr {
    fn from(value: &String) -> Self {
        Self(value.to_owned())
    }
}

impl Into<String> for Expr {
    fn into(self) -> String {
        self.0.to_string()
    }
}
