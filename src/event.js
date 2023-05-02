let events = [];

function event(name, data) {
  events.push({ name, data: JSON.stringify(data) });
}

function list() {
  if (events.length) {
    return events;
  }
}

function clear() {
  events = [];
}

module.exports.event = event;
module.exports.list = list;
module.exports.clear = clear;
