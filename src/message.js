let messages = [];

module.exports.message = function (process, payload) {
  messages.push({ process, payload: JSON.stringify(payload) });
};

module.exports.list = function () {
  if (messages.length) {
    return messages;
  }
};

module.exports.clear = function () {
  messages = [];
};
