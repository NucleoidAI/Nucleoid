var events = [];

module.exports.event = function(name, payload) {
  events.push({ name, payload: JSON.stringify(payload) });
};

module.exports.list = function() {
  if (events.length) {
    return events;
  }
};

module.exports.clear = function() {
  events = [];
};
