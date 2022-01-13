use chrono::prelude::*;
use serde::Deserialize;

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonClass {
    course: String,
    start_time: String,
    end_time: String,
    link: String,
}

#[derive(Debug, Clone)]
pub struct Class {
    course: String,
    start_time: NaiveTime,
    end_time: NaiveTime,
    link: Option<String>,
}

impl Class {
    pub fn course(&self) -> &str {
        &self.course
    }

    pub fn start_time(&self) -> NaiveTime {
        self.start_time
    }

    pub fn end_time(&self) -> NaiveTime {
        self.end_time
    }

    pub fn link(&self) -> &Option<String> {
        &self.link
    }
}

impl TryFrom<JsonClass> for Class {
    type Error = &'static str;
    fn try_from(j: JsonClass) -> Result<Self, Self::Error> {
        let start_time = NaiveTime::parse_from_str(&j.start_time, "%H:%M").map_err(|_| "Error parsing startTime")?;
        let end_time = NaiveTime::parse_from_str(&j.end_time, "%H:%M").map_err(|_| "Error parsing endTime")?;
        Ok(Self {
            course: j.course,
            start_time,
            end_time,
            link: Some(j.link).filter(|s| !s.is_empty()),
        })
    }
}

#[derive(Debug)]
pub struct Schedule {
    days: [Vec<Class>; 7],
}

impl Schedule {
    fn from_map(hashmap: HashMap<Weekday, Vec<Class>>) -> Self {
        let mut days: [Vec<Class>; 7] = [vec![], vec![], vec![], vec![], vec![], vec![], vec![]];
        for (weekday, classes) in hashmap {
            days[weekday as usize] = classes;
        }
        Self { days }
    }

    fn classes_on_weekday(&self, weekday: Weekday) -> &[Class] {
        &self.days[weekday as usize]
    }

    pub fn classes_on_date(&self, date: impl Datelike) -> &[Class] {
        self.classes_on_weekday(date.weekday())
    }
}

pub fn get_weekly_schedule(path: &str) -> Schedule {
    let file = File::open(path).expect("json schedule file not found");
    let reader = BufReader::new(file);

    let schedule: HashMap<Weekday, Vec<Class>> =
        serde_json::from_reader::<_, HashMap<Weekday, Vec<JsonClass>>>(reader)
            .expect("invalid schedule json")
            .into_iter()
            .map(|(weekday, vecjsonsess)| {
                let vecsess = vecjsonsess.into_iter().map(|j| j.try_into().unwrap()).collect();
                (weekday, vecsess)
            })
            .collect();

    Schedule::from_map(schedule)
}
