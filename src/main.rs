use std::{
  env,
  fmt::Display,
  fs::File,
  io::{self, BufRead},
  path::{Path, PathBuf},
};

use chrono::{DateTime, Local};
use colored::{Color, Colorize};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
struct LogLine {
  level: String,
  message: String,
  timestamp: String,
  file: Option<String>,
  line: Option<i32>,
  metadata: Option<Value>,
}

impl Display for LogLine {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let meta = metadata_to_string(&self.metadata);
    let time = to_local_time(&self.timestamp);

    if self.file.is_none() || self.line.is_none() {
      write!(
        f,
        "{t}|{l:^5}: {msg} {meta}",
        t = time.magenta(),
        l = self.level,
        msg = self.message,
        meta = meta,
      )
    } else {
      write!(
        f,
        "{t}|{l:^5}|{file}:{line}: {msg} {meta}",
        t = time.magenta(),
        l = self.level,
        file = self.file.clone().unwrap().blue(),
        line = self.line.unwrap().to_string().blue(),
        msg = self.message,
        meta = meta,
      )
    }
  }
}

fn to_local_time(timestamp: &str) -> String {
  if let Ok(time) = DateTime::parse_from_rfc3339(timestamp) {
    let time: DateTime<Local> = time.into();
    time.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
  } else {
    timestamp.to_string()
  }
}

fn metadata_to_string(metadata: &Option<Value>) -> String {
  if let Some(meta) = metadata {
    if let Ok(s) = serde_json::to_string(&meta) {
      return s;
    }
  }

  String::new()
}

// The output is wrapped in a Result to allow matching on errors.
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
  P: AsRef<Path>,
{
  let file = File::open(filename)?;
  Ok(io::BufReader::new(file).lines())
}

fn get_color(level: &str) -> Option<Color> {
  match level {
    "info" => Some(Color::Green),
    "warn" => Some(Color::Yellow),
    "error" => Some(Color::Red),
    "debug" => Some(Color::Cyan),
    _ => None,
  }
}

fn main() {
  let filename = env::args().nth(1);

  if filename.is_none() {
    eprintln!("No input.");
    return;
  }

  let filename = PathBuf::from(filename.unwrap());

  if let Ok(lines) = read_lines(filename) {
    for line in lines.flatten() {
      if let Ok(v) = serde_json::from_str::<LogLine>(line.as_str()) {
        if let Some(color) = get_color(&v.level) {
          println!("{}", v.to_string().color(color));
        } else {
          println!("{}", v);
        }
      } else {
        println!("{}", line);
      }
    }
  }
}
