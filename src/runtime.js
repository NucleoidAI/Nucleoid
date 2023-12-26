require("./prototype");
const datastore = require("./datastore");
const stack = require("./stack");
const Statement = require("./statement");
const Event = require("./event");
const transaction = require("./transaction");
const config = require("./config");
let eventExtension;

try {
  const { path } = config();
  eventExtension = require(`${path}/extensions/event.js`);
} catch (err) {} // eslint-disable-line no-empty

module.exports.process = function (string, options = {}) {
  options = { ...config(), ...options };
  const { declarative, details } = options;

  const before = Date.now();
  let result;

  try {
    const statements = Statement.compile(string, options);
    transaction.start();
    result = stack.process(statements, null, options);
    transaction.end();
  } catch (error) {
    transaction.rollback();
    result = { ...result, error, value: error.toString() };
  }

  const events = Event.list();
  const date = Date.now();
  const time = date - before;

  datastore.write({
    s: string,
    $: result.$nuc,
    c: declarative ? true : undefined,
    t: time,
    r: result.value,
    d: date,
    e: result.error ? true : undefined,
    v: events,
  });

  if (eventExtension) {
    eventExtension.apply(events);
  }

  Event.clear();

  if (details) {
    return {
      result: result.value,
      $nuc: result.$nuc,
      date,
      time,
      error: !!result.error,
      events,
    };
  } else {
    if (result.error) {
      throw result.error;
    }

    return result.value;
  }
};
