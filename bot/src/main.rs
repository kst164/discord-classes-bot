use chrono::prelude::*;
use chrono::Duration as OldDuration;

use std::fs::OpenOptions;
use std::io::Write;

mod sched;
use sched::{Class, get_weekly_schedule};
mod webhook_manager;
use webhook_manager::WebhookManager;

#[tokio::main]
async fn main() {
    let mut webhook_manager = WebhookManager::new(env!("DISCORD_WEBHOOK_URI").into());
    loop {
        webhook_manager.delete_all().await; // Just in case idk
        main_today(&mut webhook_manager).await;

        let two_tomorrow = Local::today().succ().and_hms(2, 0, 0);
        tokio::time::sleep((two_tomorrow - Local::now()).to_std().unwrap()).await;
    }
}


async fn main_today(webhook_manager: &mut WebhookManager) {
    let mut file = OpenOptions::new().write(true).append(true).create(true).open("test_log.txt").unwrap();

    let today = Local::today();

    let schedule = get_weekly_schedule("../weekly.json");

    let upcoming_time = chrono::Duration::minutes(15);

    //let events_today = events(schedule.classes_on_weekday(today.weekday()), upcoming_time);
    let events_today = events(schedule.classes_on_date(today), upcoming_time);

    writeln!(file, "{}: Start", Local::now()).unwrap();

    for (instant, event, class) in events_today.into_iter().map(|(t, ev, cl)| (today.and_time(t).unwrap(), ev, cl)) {
        let diff = instant - Local::now();
        if -diff > OldDuration::minutes(15) {
            // More than 10 minutes ago, probably missed it
            writeln!(file, "{}: Skipping {:?} of {}", Local::now(), event, class.course()).unwrap();
            continue;
        }

        if diff > OldDuration::zero() {
            writeln!(file, "{}: Waiting for {:?} of {}", Local::now(), event, class.course()).unwrap();
            tokio::time::sleep(diff.to_std().unwrap()).await;
        }

        writeln!(file, "{}: {} is {:?}", Local::now(), class.course(), event).unwrap();

        match event {
            Event::Upcoming => {
                webhook_manager.send_upcoming(class).await;
            },
            Event::Starting => {
                webhook_manager.set_starting(class).await;
            },
            Event::Ending => {
                webhook_manager.delete(class).await;
            }
        }
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
