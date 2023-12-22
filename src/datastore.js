const revive = require("./lang/$nuc/revive");
let datastore;

function init(config = { cache: true }) {
  if (config.cache) {
    datastore = require("./cache");
  } else {
    datastore = require("@nucleoidjs/datastore");
  }
}

function clear() {
  datastore.clear();
}

function read() {
  return datastore
    .read()
    .map((statement) => ({ ...statement, $: revive(statement.$) }));
}

function write(data) {
  return datastore.write(data);
}

function tail(n) {
  return datastore.tail(n);
}

module.exports = { init, read, write, clear, tail };
