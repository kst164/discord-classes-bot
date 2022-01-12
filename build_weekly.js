const slots_template = require('./slots_template.json');
const course_slots = require('./course_slots.json');

const days = ['Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday'];

async function main() {
  const weekly = {};

  for (const day of days) {
    weekly[day] = [];
  }

  for (const course in course_slots) {
    const slot = course_slots[course];
    for (const session of slots_template[slot]) {
      weekly[session.day].push({
        course: course,
        startTime: session.startTime,
        endTime: session.endTime,
        link: "",
      });
    }
  }

  for (const day of days) {
    weekly[day].sort((a, b) => a.startTime < b.startTime ? -1 : 1);
  }

  console.log(JSON.stringify(weekly, null, 2));
}

main();
