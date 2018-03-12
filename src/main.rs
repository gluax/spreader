extern crate chrono;
extern crate regex;
extern crate reqwest;
extern crate rss;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate select;

use select::document::Document;
use select::predicate::{Attr, Class, Name, Predicate};
use chrono::prelude::*;
use std::fs::File;
use std::io::{Read, Write};
use regex::Regex;
use rss::Channel;
//use serde::Deserialize;
//use serde::de::Deserializer;

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

// #[derive(Debug)]
// enum ActionType {
//     ReadFeed,
//     Open,
//     Find,
//     Filter,
//     Get,
//     ToFileFormat,
//     WriteToFile,
// }

#[derive(Clone, Debug)]
enum TaskType {
    Dom(select::document::Document),
    Feed(rss::Channel),
    Text(String),
}

impl TaskType {
    fn dom(self) -> Option<select::document::Document> {
        if let TaskType::Dom(d) = self {
            Some(d)
        } else {
            None
        }
    }

    fn feed(self) -> Option<rss::Channel> {
        if let TaskType::Feed(f) = self {
            Some(f)
        } else {
            None
        }
    }

    fn text(self) -> Option<String> {
        if let TaskType::Text(t) = self {
            Some(t)
        } else {
            None
        }
    }
}

#[derive(Clone, Deserialize, Debug)]
struct Task {
    name: String,
    task_type: String,
    selector: Option<String>,
    selector_attr: Option<String>,
    selector_body: Option<bool>,
    filter: Option<String>,
    open_url: Option<bool>,
    match_filename: Option<String>,
    output_concat: Option<String>,
}

impl Task {
    fn perform(&self, output_path: &str, data: Vec<TaskType>, add: Option<Vec<TaskType>>) -> (Vec<TaskType>, Option<Vec<TaskType>>) {
        let mut rsp: Vec<TaskType> = Vec::new();
        let mut additional: Vec<TaskType> = Vec::new();

        //for now all tasktypes are hardcoded will fix in future
        if self.name.contains("read feed") {
            rsp = read_feed(data.clone(), "feeds/last_de");
        }
        if self.name.contains("get") {
            additional = get(data.clone(), &self.clone().match_filename.unwrap());
        }
        if self.name.contains("to file format") {
            rsp = file_format(data.clone(), &self.selector.clone().unwrap(), self.selector_body.clone().unwrap(), &self.output_concat.clone().unwrap())
        }
        if self.name.contains("open") {
            rsp = open(data.clone(), self.open_url.unwrap())
        }

        (rsp, Some(additional))
    }
}

fn read_config() -> Config {
    let mut conf = File::open("Config.toml").expect("Error opening config");
    let mut buf = Vec::new();
    conf.read_to_end(&mut buf).expect("Error reading config");

    toml::from_str(String::from_utf8(buf).unwrap().as_ref()).expect("Invalid Config Format")
}

fn open_url(url: &str) -> Result<select::document::Document, reqwest::Error> {
    Ok(Document::from(get_req(url)?.as_ref()))
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

fn read_feed(data: Vec<TaskType>, tracker_path: &str) -> Vec<TaskType>{
    
    //read last date from tracker file
    let last_update = DateTime::parse_from_rfc2822(&read_tracker(tracker_path)).unwrap().with_timezone(&FixedOffset::east(0));
    
    //read the feed url content
    let feed_content = data[0].clone().feed().unwrap();
    
    //for each item in feed see if the date of the chapters are greater than our date

    let mut urls: Vec<TaskType> = Vec::new();
    for channel in feed_content.items() {
        let chapter_pub_date = DateTime::parse_from_rfc2822(channel.pub_date().unwrap()).unwrap();
        //if latest chapter date is newer 
        if chapter_pub_date > last_update {
            //perform next task on url || return list of them?
            urls.push(TaskType::Text(channel.link().unwrap().to_string()));
            println!("pubDate: {:?}, Chapter: {}", chapter_pub_date, channel.title().unwrap());
        } 
    }

    urls
}

fn get(data: Vec<TaskType>, matchers: &str) -> Vec<TaskType> {
    let mut filenames: Vec<TaskType> = Vec::new();
    let matcher = Regex::new(matchers).unwrap();
    for ttype in data {
        let link = ttype.clone().text().unwrap();
        let matched = matcher.find(&link).unwrap();
        let filename = link[matched.start()..].to_string();
        filenames.push(TaskType::Text(filename));
    }

    filenames
}

fn open(data: Vec<TaskType>, open: bool) -> Vec<TaskType> {
    let mut doms: Vec<TaskType> = Vec::new();
    
    for link in data {
        if open {
            let dom = open_url(&link.text().unwrap()).unwrap();
            //println!("dom: {:?}", dom);
            doms.push(TaskType::Dom(dom));
        }
    }
    
    doms
}

fn file_format(data: Vec<TaskType>, selector: &str, selector_body: bool, output_concat: &str) -> Vec<TaskType> {
    let mut chapters: Vec<TaskType> = Vec::new();

    let mut chapter = "".to_owned();
    let concat = output_concat.to_owned();
    
    for wdom in data {
       
        if selector_body {
            
            let dom = wdom.clone().dom().unwrap();
            // temp harded coded till we write a parser for selector
            for node in dom.find(Class("section-content").descendant(Name("p"))) {
                chapter.push_str(&node.text().to_owned());
                chapter.push_str(&concat);
            }

            chapters.push(TaskType::Text(chapter.clone()));
            chapter.clear();
        }
    }

    chapters
}

fn write(output_path: &str, data: Vec<TaskType>, add: Vec<TaskType>) {
    
}

fn main() {
    let conf: Config = read_config();
    for feed in &conf.feed {
        let mut data: Vec<TaskType> = vec![TaskType::Feed(Channel::from_url(&feed.feed_url).unwrap())];
        let mut add: Option<Vec<TaskType>> = None;
        //let mut tasks = feed.task.clone();
        //let mut s: &str = &get_req("http://www.wuxiaworld.com/novel/desolate-era/de-book-42-chapter-20").unwrap();
        //let dom = Document::from(s);
        //for node in dom.find(Class("section-content").descendant(Name("p"))) {
            //println!("text: {}", node.text());
        //}
        
        println!("feed: {:?}", feed);
        
        for task in feed.task.clone() {
            println!("task: {:?}", task);
            let rsp = task.perform(&feed.output_path.clone(), data.clone(), add.clone());
            data = rsp.0;
            add = rsp.1;
        }
    }

}
