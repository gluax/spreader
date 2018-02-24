//extern crate regex;
extern crate reqwest;
extern crate rss;
extern crate scraper;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;

use std::fs::File;
use std::io::{Read, Write};
//use regex::Regex;
//use rss::Channel;
use scraper::Html;
use serde::Deserialize;
use serde::de::Deserializer;

#[derive(Deserialize, Debug)]
struct Config {
  feed: Vec<Feed>,
}

#[derive(Deserialize, Debug)]
struct Feed {
  feed_url: String,
  feed_type: String,
  tracker: String,
  output_path: String,
  task: Vec<Task>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum Action {
  ReadFeed,
  OpenHome,
  FindChapUrls,
  FilterUrls,
  GetFilenameAndOpenChapter,
  ChapterToFileFormat,
  WriteToFile,
}

fn deserialize_action<'de, D>(deserializer: D) -> Result<Action, D::Error> where D: Deserializer<'de> {
  let s = String::deserialize(deserializer)?;

  match s.as_ref() {
    "read feed" => Ok(Action::ReadFeed),
    "open chapter homepage" => Ok(Action::OpenHome),
    "find chapter urls" => Ok(Action::FindChapUrls),
    "filter out bad urls" => Ok(Action::FilterUrls),
    "get filename and open chapter link" => Ok(Action::GetFilenameAndOpenChapter),
    "get chapter content to file format" => Ok(Action::ChapterToFileFormat),
    "write to file" => Ok(Action::WriteToFile),
    _ => Err(serde::de::Error::custom("Error trying to deserialize Action policy config"))
  }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum TaskType {
  Dom,
  File,
  Text,
}

fn deserialize_tasktype<'de, D>(deserializer: D) -> Result<TaskType, D::Error> where D: Deserializer<'de> {
  let s = String::deserialize(deserializer)?;

  match s.as_ref() {
    "dom" => Ok(TaskType::Dom),
    "file" => Ok(TaskType::File),
    "text" => Ok(TaskType::Text),
    _ => Err(serde::de::Error::custom("Error trying to deserialize TaskType policy config"))
  }
}

#[derive(Deserialize, Debug)]
struct Task {
  #[serde(deserialize_with="deserialize_action")]
  name: Action,
  #[serde(deserialize_with="deserialize_tasktype")]
  task_type: TaskType, //turn this into an enum
  selector: Option<String>,
  selector_attr: Option<String>,
  selector_body: Option<bool>,
  filter: Option<String>,
  open_url: Option<bool>,
  match_filename: Option<String>,
  output_concat: Option<String>,
}

fn read_config() -> Config {
  let mut conf = File::open("Config.toml").expect("Error opening config");
  let mut buf = Vec::new();
  conf.read_to_end(&mut buf).expect("Error reading config");

  return toml::from_str(String::from_utf8(buf).unwrap().as_ref()).expect("Invalid Config Format");
}

fn open_url(url: &str) -> Result<Html, reqwest::Error> {
  Ok(Html::parse_document(get_req(url)?.as_ref()))
}

fn get_req(uri: &str) -> Result<String, reqwest::Error> {
  Ok(reqwest::get(uri)?.text()?)
}

fn write_tracker<T: std::string::ToString>(path: &str, index: T) {
  let f = &mut File::create(path).expect(format!("Error creating {}", path).as_ref());
  f.write(index.to_string().as_bytes().as_ref())
    .expect(format!("Error writing to {}", path).as_ref());
  f.sync_all().expect("Error syncing to disk");
}

fn read_tracker(path: &str) -> i32 {
  if let Ok(mut f) = File::open(path) {
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)
      .expect(format!("Error reading {}", path).as_ref());
    if let Ok(res) = String::from_utf8(buf).unwrap().parse::<i32>() {
      res
    } else {
      -1
    }
  } else {
    -1
  }
}

fn main() {
  let conf: Config = read_config();

  for feed in &conf.feed {
    println!("feed: {:?}", feed);
    
  }

}
