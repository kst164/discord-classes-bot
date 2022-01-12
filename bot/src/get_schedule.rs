use chrono::prelude::*;
use serde::Deserialize;

use std::fs::File;
use std::io::BufReader;

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct JsonClass {
    pub course: String,
    pub startTime: String,
    pub endTime: String,
    pub link: String,
}

/// A day of the week, and all the classes on that day
#[derive(Deserialize, Debug)]
pub struct Day {
    pub weekday: Weekday,
    pub classes: Vec<JsonClass>,
}

#[derive(Debug)]
pub struct Session {
    pub weekday: Weekday,
    pub course: String,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub link: Option<String>,
}

impl Session {
    fn from_json_and_day(jc: JsonClass, weekday: Weekday) -> Self {
        Self {
            weekday,
            course: jc.course,
            start_time: NaiveTime::parse_from_str(&jc.startTime, "%H:%M").unwrap(),
            end_time: NaiveTime::parse_from_str(&jc.endTime, "%H:%M").unwrap(),
            link: Some(jc.link).filter(|l| !l.is_empty()),
        }
    }
}

pub fn get_schedule(path: &str) -> Vec<Session> {
    let file = File::open(path).expect("json schedule file not found");
    let reader = BufReader::new(file);

    let schedule: Vec<Day> = serde_json::from_reader(reader).expect("invalid schedule json");

    let classes = schedule.into_iter().flat_map(|day| {
        day.classes
            .into_iter()
            .map(move |class| Session::from_json_and_day(class, day.weekday))
    });

    classes.collect()
}
