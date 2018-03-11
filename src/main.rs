extern crate chrono;
extern crate regex;
extern crate reqwest;
extern crate rss;
extern crate scraper;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;

use chrono::prelude::*;
use std::fs::File;
use std::io::{Read, Write};
use regex::Regex;
use rss::Channel;
use scraper::{Html, Selector};
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

#[derive(Clone, Deserialize, Debug)]
enum ActionType {
    ReadFeed,
    Open,
    Find,
    Filter,
    Get,
    ToFileFormat,
    WriteToFile,
}

fn deserialize_action<'de, D>(deserializer: D) -> Result<ActionType, D::Error> where D: Deserializer<'de> {
    let s = String::deserialize(deserializer)?;

    match s.as_ref() {
        "read feed" => Ok(ActionType::ReadFeed),
        "open" => Ok(ActionType::Open),
        "find" => Ok(ActionType::Find),
        "filter" => Ok(ActionType::Filter),
        "get" => Ok(ActionType::Get),
        "format" => Ok(ActionType::ToFileFormat),
        "write" => Ok(ActionType::WriteToFile),
        _ => Err(serde::de::Error::custom("Error trying to deserialize Action policy config"))
    }
}

#[derive(Clone, Deserialize, Debug)]
enum TaskType {
    Dom(String),
    File(String),
    Text(String),
}

// fn deserialize_tasktype<'de, D>(deserializer: D) -> Result<TaskType, D::Error> where D: Deserializer<'de> {
//     let s = String::deserialize(deserializer)?;

//     match s.as_ref() {
//         "dom" => Ok(TaskType::Dom),
//         "file" => Ok(TaskType::File),
//         "text" => Ok(TaskType::Text),
//         _ => Err(serde::de::Error::custom("Error trying to deserialize TaskType policy config"))
//     }
// }

#[derive(Clone, Deserialize, Debug)]
struct Task {
    //#[serde(deserialize_with="deserialize_action")]
    name: String,
    //#[serde(deserialize_with="deserialize_tasktype")]
    task_type: String, //turn this into an enum
    selector: Option<String>,
    selector_attr: Option<String>,
    selector_body: Option<bool>,
    filter: Option<String>,
    open_url: Option<bool>,
    match_filename: Option<String>,
    output_concat: Option<String>,
}

impl Task {
    fn perform(&self, data: TaskType) -> TaskType {
        TaskType::Dom("cry".to_string())
    }
}

fn read_config() -> Config {
    let mut conf = File::open("Config.toml").expect("Error opening config");
    let mut buf = Vec::new();
    conf.read_to_end(&mut buf).expect("Error reading config");

    toml::from_str(String::from_utf8(buf).unwrap().as_ref()).expect("Invalid Config Format")
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

fn read_tracker(path: &str) -> String {
    if let Ok(mut f) = File::open(path) {
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)
            .expect(format!("Error reading {}", path).as_ref());
        if let Ok(mut res) = String::from_utf8(buf) {
            res.pop();
            res
        } else {
            "invalid datetime".to_string()
        }
    } else {
        "failed to open path".to_string()
    }
}

// so maybe a list of links or dom or etc
// data the type of what's being passed from previous stuff, task the current task, the list of tasks to still be performed
fn perform_task(data: TaskType, task: &Task, tasks: &mut Vec<Task>) {
    //somehow do the thing to make it do the right thing
    // task.name tells us what action to do open, filter, write, etc
    
}

fn read_feed(url: &str, output_path: &str, tracker_path: &str, tasks: &mut Vec<Task>) {
    let (_, tasks) = tasks.split_at_mut(1);
    
    //read last date from tracker file
    let last_update = DateTime::parse_from_rfc2822(&read_tracker(tracker_path)).unwrap().with_timezone(&FixedOffset::east(0));
    
    //read the feed url content
    let feed_content = Channel::from_url(url).unwrap();
    
    //for each item in feed see if the date of the chapters are greater than our date
    for channel in feed_content.items() {
        //grab latest chapter as date
        let chapter_pub_date = DateTime::parse_from_rfc2822(channel.pub_date().unwrap()).unwrap();
        //if latest chapter date is newer 
        if chapter_pub_date > last_update {
            //perform next task on url || return list of them? 
            println!("pubDate: {:?}, Chapter: {}", chapter_pub_date, channel.title().unwrap());
            //open_chap_home(&channel.link().unwrap(), output_path, &mut tasks.to_vec());
        } 
    }
    
}

fn open_chap_home(url: &str, output_path: &str, tasks: &mut Vec<Task>)  {
    let (task, tasks) = tasks.split_at_mut(1);
    
    if task[0].open_url.unwrap() {
        find_chap_urls(output_path, open_url(url).unwrap(), &mut tasks.to_vec());
    }
}

fn find_chap_urls(output_path: &str, dom: scraper::Html, tasks: &mut Vec<Task>) {
    let (task, tasks) = tasks.split_at_mut(1);
    let selector = Selector::parse(&task[0].clone().selector.unwrap()).unwrap();
    
    for e in dom.select(&selector) {
        filter_urls(e.value().attr(&task[0].clone().selector_attr.unwrap()).unwrap(), output_path, &mut tasks.to_vec());
    }
}

fn filter_urls(url: &str, output_path: &str, tasks: &mut Vec<Task>) {
    let (task, tasks) = tasks.split_at_mut(1);
    let filter = Regex::new(&task[0].clone().filter.unwrap()).unwrap();

    if filter.is_match(url) {
        get_filename_and_open_link(url, output_path, &mut tasks.to_vec());
    }
}

fn get_filename_and_open_link(url: &str, output_path: &str, tasks: &mut Vec<Task>) {
    let (task, _) = tasks.split_at_mut(1);
    let match_filename = Regex::new(&task[0].clone().match_filename.unwrap()).unwrap();
    
    let mat = match_filename.find(url).unwrap();
    let filename = &url[mat.start()..];
    
    let dom = open_url(url).unwrap();
    get_chapter_content_to_file_format(filename, output_path, dom);
}

fn get_chapter_content_to_file_format(filename: &str, output_path: &str, dom: scraper::Html) {

    let selector = Selector::parse("div[itemprop=\"articleBody\"] p").unwrap();
    let mut chapter_paras: Vec<String> = Vec::new();
    
    for content in dom.select(&selector) {
        chapter_paras.push(content.inner_html());
    }

}

fn main() {
    let conf: Config = read_config();
    for feed in &conf.feed {
        let mut tasks = feed.task.clone();
        println!("feed: {:?}", feed);
        feed.task[0].perform(TaskType::Text(feed.feed_url.clone()));
        read_feed(feed.feed_url.as_ref(), feed.output_path.as_ref(), feed.tracker.as_ref(), &mut tasks);
    }

}
