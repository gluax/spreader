extern crate regex;
extern crate reqwest;
extern crate rss;
extern crate scraper;
#[macro_use]
extern crate serde_derive;
extern crate toml;

use std::fs::File;
use std::io::{Read, Write};
use regex::Regex;
use rss::Channel;
use scraper::Html;

#[derive(Deserialize)]
struct Config {
  feeds: Vec<Feed>,
}

#[derive(Deserialize)]
struct Feed {
  url: String,
  chapter_regex: String,
  tracker: String,
  page_selector: String,
  link_selector: String,
  text_selector: String,
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

fn format<'a>(read: &'a mut str, re: Regex, link: &str) -> &'a str {
  let mut pos:usize = 0;
 
  for capture in re.captures_iter(link) {
    println!("{}", read.replacen("replace", &capture[pos], 1));
    println!("re: {}", read);
    pos += 1;
  }

  read
  
}

fn main() {
  let conf: Config = read_config();

  for feed in &conf.feeds {
    
    let index_regex = Regex::new(feed.regex.as_ref()).expect("Invalid feed regex");
    let last = read_tracker(feed.tracker.as_ref());
    let xml = Channel::from_url(&feed.url).unwrap();
    let first_link = xml.items()[0].link().unwrap();
    
    //let mut text = get_req(format(feed.link, index_regex, first_link)).unwrap();
    
    
    
    println!("{}\n {}\n {:?}", index_regex, last, first_link);
    println!("reformated: {}", format(read, index_regex, first_link));
    
  }

}
