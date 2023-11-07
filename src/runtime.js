const datastore = require("@nucleoidjs/datastore");
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
  const { declarative, details, cacheOnly } = options;

  const before = Date.now();
  let result;

  try {
    const statements = Statement.parse(string, options);
    transaction.start();
    result = stack.process(statements, null, options);
    transaction.end();
  } catch (error) {
    transaction.rollback();
    result = { ...result, error };
  }

  const events = Event.list();
  const date = Date.now();
  const time = date - before;

  if (result.error) {
    result.value = `${result.error.constructor.name}: ${result.error.message}`;
  }

  if (!cacheOnly) {
    datastore.write({
      s: string,
      $: JSON.stringify(result.$nuc),
      c: declarative ? true : undefined,
      t: time,
      r: result.value,
      d: date,
      e: result.error,
      v: events,
    });
  }

  if (eventExtension) {
    // TODO Disable for restart
    eventExtension.apply(events);
  }

  Event.clear();

  if (details) {
    return {
      result: result.value,
      $nuc: result.$nuc,
      date,
      time,
      error: result.error,
      events,
    };
  } else {
    if (result.error) {
      throw result.error;
    }

    return result.value;
  }
};
