let messages = [];

function message(process, payload) {
  messages.push({ process, payload: JSON.stringify(payload) });
}

function list() {
  if (messages.length) {
    return messages;
  }
}

function clear() {
  messages = [];
}

module.exports.message = message;
module.exports.list = list;
module.exports.clear = clear;
