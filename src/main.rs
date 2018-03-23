extern crate chrono;
extern crate regex;
extern crate reqwest;
extern crate rss;
extern crate scraper;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;

use scraper::{Html, Selector};
use chrono::prelude::*;
use std::fs::File;
use std::io::{Read, Write};
use regex::Regex;
use rss::Channel;
use std::cell::RefCell;
use std::rc::Rc;

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

#[derive(Clone, Debug)]
enum TaskType {
    Dom(Html),
    Feed(rss::Channel),
    Text(String),
}

impl From<Html> for TaskType {
    fn from(info: Html) -> Self {
        TaskType::Dom(info)
    }
}

impl From<Channel> for TaskType {
    fn from(info: Channel) -> Self {
        TaskType::Feed(info)
    }
}

impl From<String> for TaskType {
    fn from(info: String) -> Self {
        TaskType::Text(info)
    }
}

impl TaskType {
    fn dom(self) -> Option<Html> {
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
    selector: Option<String>,
    selector_attr: Option<String>,
    filter: Option<String>,
    match_filename: Option<String>,
    output_concat: Option<String>,
    open_url: Option<bool>,
    selector_body: Option<bool>,
    get: Option<bool>,
    feed: Option<bool>,
    write: Option<bool>,
}

impl Task {
    fn perform(
        &self,
        tracker: &str,
        output_path: &str,
        data: Vec<TaskType>,
        add: Option<Vec<TaskType>>,
    ) -> (Vec<TaskType>, Option<Vec<TaskType>>) {
        let mut rsp: Vec<TaskType> = Vec::new();
        let mut addi: Vec<TaskType> = Vec::new();
        if let Some(a) = add {
            addi = a;
        }

        if let Some(true) = self.feed {
            rsp = read_feed(data.clone(), tracker);
        }
        if let Some(true) = self.get {
            addi = get(data.clone(), &self.clone().match_filename.unwrap());
        }
        if let Some(true) = self.selector_body {
            rsp = file_format(
                data.clone(),
                &self.selector.clone().unwrap(),
                &self.output_concat.clone().unwrap(),
            );
        }
        if let Some(true) = self.open_url {
            rsp = open(data.clone());
        }
        if let Some(true) = self.write {
            write_chapter(output_path, data.clone(), addi.clone());
        }

        (rsp, Some(addi))
    }
}

fn read_config() -> Config {
    let mut conf = File::open("Config.toml").expect("Error opening config");
    let mut buf = Vec::new();
    conf.read_to_end(&mut buf).expect("Error reading config");

    toml::from_str(String::from_utf8(buf).unwrap().as_ref()).expect("Invalid Config Format")
}

fn open_url(url: &str) -> Result<Html, reqwest::Error> {
    //Ok(Document::from(get_req(url)?.as_ref()))
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

fn read_feed(data: Vec<TaskType>, tracker_path: &str) -> Vec<TaskType> {
    //read last date from tracker file
    let last_update = DateTime::parse_from_rfc2822(&read_tracker(tracker_path))
        .unwrap()
        .with_timezone(&FixedOffset::east(0));

    //read the feed url content
    let feed_content = data[0].clone().feed().unwrap();

    //for each item in feed see if the date of the chapters are greater than our date

    let mut urls: Vec<TaskType> = Vec::new();
    for channel in feed_content.items() {
        let chapter_pub_date = DateTime::parse_from_rfc2822(channel.pub_date().unwrap()).unwrap();
        //if latest chapter date is newer
        if chapter_pub_date > last_update {
            //perform next task on url || return list of them?
            urls.push(TaskType::from(channel.link().unwrap().to_string()));
            //println!("pubDate: {:?}, Chapter: {}", chapter_pub_date, channel.title().unwrap());
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
        filenames.push(TaskType::from(filename));
    }

    filenames
}

fn open(data: Vec<TaskType>) -> Vec<TaskType> {
    let mut doms: Vec<TaskType> = Vec::new();

    for link in data {
        let dom = open_url(&link.text().unwrap()).unwrap();
        //println!("dom: {:?}", dom);
        doms.push(TaskType::from(dom));
    }

    doms
}

fn file_format(data: Vec<TaskType>, selector: &str, output_concat: &str) -> Vec<TaskType> {
    let mut chapters: Vec<TaskType> = Vec::new();

    let mut chapter = "".to_owned();
    let concat = output_concat.to_owned();

    let sel = Selector::parse(selector).unwrap();

    for wdom in data {
        let dom = wdom.clone().dom().unwrap();
        for chap in dom.select(&sel) {
            chapter.push_str(&chap.inner_html());
            chapter.push_str(&concat);
        }
        chapters.push(TaskType::from(chapter.clone()));
        chapter.clear();
    }

    chapters
}

fn write_chapter(output_path: &str, data: Vec<TaskType>, add: Vec<TaskType>) {
    //println!("add: {:?}", add);

    for info in data.iter().zip(add.iter()) {
        let (chap, file) = info;

        let mut path = output_path.to_owned();
        path.push_str(&file.clone().text().unwrap().to_owned());

        let mut file =
            File::create(path.clone()).expect(format!("Error creating {}", path.clone()).as_ref());
        file.write_all(chap.clone().text().unwrap().as_bytes())
            .expect(format!("Error writing to {}", path.clone()).as_ref());
        file.sync_all().expect("Error syncing to disk");
    }
}

fn main() {
    let conf: Config = read_config();
    for feed in &conf.feed {
        let mut data: Vec<TaskType> =
            vec![TaskType::Feed(Channel::from_url(&feed.feed_url).unwrap())];
        let mut add: Option<Vec<TaskType>> = None;
        let tracker = Rc::new(RefCell::new(&feed.tracker));
        let output_path = Rc::new(RefCell::new(&feed.output_path));

        //println!("feed: {:?}", feed);

        for task in feed.task.clone() {
            //println!("task: {:?}", task);
            let rsp = task.perform(&tracker.borrow(), &output_path.borrow(), data, add);
            data = rsp.0;
            add = rsp.1;
        }
    }
}
