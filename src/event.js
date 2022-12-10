let events = [];

function event(name, payload) {
  events.push({ name, payload: JSON.stringify(payload) });
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
