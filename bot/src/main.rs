use chrono::prelude::*;
use chrono::Duration as OldDuration;

use std::fs::OpenOptions;
use std::io::Write;
use std::time::Duration;

//mod get_schedule;
mod sched;
use sched::{Class, get_weekly_schedule};

#[tokio::main]
async fn main() {
    let mut file = OpenOptions::new().write(true).append(true).create(true).open("test_log.txt").unwrap();

    let today = Local::today();

    let schedule = get_weekly_schedule("../weekly.json");

    let upcoming_time = chrono::Duration::minutes(15);

    //let events_today = events(schedule.classes_on_weekday(today.weekday()), upcoming_time);
    let events_today = events(schedule.classes_on_date(today), upcoming_time);

    writeln!(file, "{}: Start", Local::now()).unwrap();

    for (instant, event, class) in events_today.into_iter().map(|(t, ev, cl)| (today.and_time(t).unwrap(), ev, cl)) {
        if instant < Local::now() {
            writeln!(file, "{}: Skipping {:?} of {}", Local::now(), event, class.course()).unwrap();
            continue;
        }

        while instant > Local::now() {
            writeln!(file, "{}: Waiting for {:?} of {}", Local::now(), event, class.course()).unwrap();
            tokio::time::sleep(Duration::from_secs(5 * 60)).await; // Wait 5 minutes, check again
        }
        writeln!(file, "{}: {} is {:?}", Local::now(), class.course(), event).unwrap();
    }
}

#[derive(Debug)]
enum Event {
    Upcoming,
    Starting,
    Ending,
}

/// Returns ordered list of all events on a day, given the timings
fn events(classes: &[Class], upcoming_time: OldDuration) -> Vec<(NaiveTime, Event, &Class)> {
    let mut list = Vec::with_capacity(classes.len() * 3);
    for class in classes {
        list.push((class.start_time() - upcoming_time, Event::Upcoming, class));
        list.push((class.start_time(), Event::Starting, class));
        list.push((class.end_time(), Event::Ending, class));
    }
    list.sort_by_key(|(t, _, _)| *t);
    list
}
