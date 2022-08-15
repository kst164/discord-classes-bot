A Discord webhook based notifier that pings me 15 minutes before every class. The bot (code in
`./bot/`) is written in Rust, and reads the timetable from `weekly.js`.  `weekly.json` can be
generated using `build_weekly.js`, which uses `slots_template.json` (based on [this page](https://iith.ac.in/academics/assets/files/timetables/Timetable-Template.pdf)) and
`course_slots.json` (where the slot for each running course must be specified) to build a base
`weekly.json` file.  If a class is online, links have to added to `weekly.json` for each class.
