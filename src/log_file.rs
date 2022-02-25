use chrono::prelude::*;
use regex::Regex;
use serde::Serialize;
use std::str::Lines;

const START_PATTERN: &str = r"^\d{2}/\d{2}/\d{4}\s\d{2}:\d{2}:\d{2}";
const END_PATTERN: &str = r"^\w+\s-+\sEND\sTRANSMITTED";
const ID_PATTERN: &str = r"^(\w+)\s-+\sTRANSMITTED\s-+\s";
const ASSIGN_PATTERN: &str = r"(\w+)\s=\s(\S+)";
const DATE_FORMAT: &str = "%m/%d/%Y %H:%M:%S";

#[derive(Debug, Serialize)]
pub struct LogEntry {
    pub id: String,
    pub message_type: String,
    pub remote_id: String,
    pub server: String,
    pub timestamp: DateTime<Local>,
}

struct LogEntryBuilder {
    id: Option<String>,
    message_type: Option<String>,
    remote_id: Option<String>,
    server: Option<String>,
    timestamp: Option<DateTime<Local>>,
}

impl LogEntryBuilder {
    fn new() -> Self {
        Self {
            id: None,
            message_type: None,
            remote_id: None,
            server: None,
            timestamp: None,
        }
    }

    fn id(&mut self, id: String) {
        self.id = Some(id);
    }

    fn message_type(&mut self, message_type: String) {
        self.message_type = Some(message_type);
    }

    fn remote_id(&mut self, remote_id: String) {
        self.remote_id = Some(remote_id);
    }

    fn server(&mut self, server: String) {
        self.server = Some(server);
    }

    fn timestamp(&mut self, timestamp: DateTime<Local>) {
        self.timestamp = Some(timestamp);
    }

    fn build(self) -> LogEntry {
        LogEntry {
            id: self.id.expect("id must be present"),
            message_type: self.message_type.expect("message_type must be present"),
            remote_id: self.remote_id.expect("remote_id must be present"),
            server: self.server.expect("server must be present"),
            timestamp: self.timestamp.expect("timestamp must be present"),
        }
    }
}

pub struct LogFile<'a> {
    iter: Lines<'a>,
    start_re: Regex,
    end_re: Regex,
    id_re: Regex,
    assign_re: Regex,
}

impl<'a> LogFile<'a> {
    pub fn new(iter: Lines<'a>) -> Self {
        let start_re = Regex::new(START_PATTERN).unwrap();
        let end_re = Regex::new(END_PATTERN).unwrap();
        let id_re = Regex::new(ID_PATTERN).unwrap();
        let assign_re = Regex::new(ASSIGN_PATTERN).unwrap();

        Self {
            iter,
            start_re,
            end_re,
            id_re,
            assign_re,
        }
    }
}

impl<'a> Iterator for LogFile<'a> {
    type Item = LogEntry;

    fn next(&mut self) -> std::option::Option<<Self as Iterator>::Item> {
        let mut builder = LogEntryBuilder::new();

        while let Some(line) = self.iter.next() {
            if self.end_re.is_match(line) {
                let entry = builder.build();
                return Some(entry);
            }

            match self.start_re.captures(line) {
                Some(c) => {
                    let datetime = c.get(0).unwrap().as_str();
                    let timestamp = Local.datetime_from_str(datetime, DATE_FORMAT).unwrap();
                    builder.timestamp(timestamp);
                }
                None => {
                    if let Some(c) = self.id_re.captures(line) {
                        let id = c.get(1).unwrap().as_str().to_string();
                        builder.id(id);
                    }

                    if let Some(c) = self.assign_re.captures(line) {
                        let lhs = c.get(1).unwrap().as_str();
                        let rhs = c.get(2).unwrap().as_str().to_string();

                        match lhs {
                            "message_type" => builder.message_type(rhs),
                            "remote_id" => builder.remote_id(rhs),
                            "server" => builder.server(rhs),
                            _ => (),
                        }
                    }
                }
            }
        }

        return None;
    }
}
