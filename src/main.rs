extern crate regex;
extern crate reqwest;
extern crate scraper;
#[macro_use]
extern crate serde_derive;
extern crate toml;

use std::fs::File;
use std::io::{Read};
use regex::Regex;
use scraper::{Html};

#[derive(Deserialize)]
struct Config {
  feeds: Vec<Feed>
}

#[derive(Deserialize)]
struct Feed {
  url: String,
  regex: String,
  tracker: String,
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

fn main() {
  let conf: Config = read_config();

  for feed in &conf {
    let index_regex = Regex::new(conf.feed.regex.as_ref()).expect("Invalid feed regex");

    loop {
      
      
      
    }
    
  }
  
  println!("{} {}", conf.feeds[0].url, index_regex);
}
