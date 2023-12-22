let cache = [];

function init() {}

function read() {
  return cache.map((data) => JSON.parse(data));
}

function write(data) {
  cache.push(JSON.stringify(data));
  return data;
}

function clear() {
  cache = [];
}

function tail(n = 10) {
  return cache
    .map((data) => JSON.parse(data))
    .reverse()
    .slice(0, n);
}

module.exports = { init, read, write, clear, tail };
