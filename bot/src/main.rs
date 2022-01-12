use chrono::prelude::*;
use chrono::Duration;
use std::collections::{HashMap, BTreeMap};
use std::time::Instant;

//mod get_schedule;
mod sched;
use sched::{Class, get_weekly_schedule};

#[tokio::main]
async fn main() {
    let today = Local::today();
    let mut next_weekday = HashMap::new();

    let one_day = Duration::days(1);

    for i in 0..=6 {
        let i_later = today + one_day * i;
        next_weekday.insert(i_later.weekday(), i_later);
    }

    let schedule = std::sync::Arc::new(get_weekly_schedule("../weekly.json"));

    let upcoming_time = chrono::Duration::minutes(15);

    let events_today = events(schedule.classes_on_weekday(today.weekday()), upcoming_time);

    println!("{}: Start", Local::now());

    for (instant, event, class) in events_today.into_iter().map(|(t, ev, cl)| (today.and_time(t).unwrap(), ev, cl)) {
        let diff = instant - Local::now();
        if diff < Duration::zero() {
            println!("{}: Skipping {:?} of {}", Local::now(), event, class.course());
            continue;
        }

        println!("{}: Waiting for {:?} of {}", Local::now(), event, class.course());
        tokio::time::sleep(diff.to_std().unwrap()).await;
        println!("{}: {} is {:?}", Local::now(), class.course(), event);
    }

    //println!("{:#?}", events(schedule.classes_on_weekday(Weekday::Mon), upcoming_time));
}

#[derive(Debug)]
enum Event {
    Upcoming,
    Starting,
    Ending,
}

/// Returns ordered list of all events on a day, given the timings
fn events(classes: &[Class], upcoming_time: chrono::Duration) -> Vec<(NaiveTime, Event, &Class)> {
    let mut list = BTreeMap::new();
    for class in classes {
        list.insert(class.start_time() - upcoming_time, (Event::Upcoming, class));
        list.insert(class.start_time(), (Event::Starting, class));
        list.insert(class.end_time(), (Event::Ending, class));
    }
    list.into_iter().map(|(t, (ev, cl))| (t, ev, cl)).collect()
}
