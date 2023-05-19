const datastore = require("@nucleoidjs/datastore");
const stack = require("./stack");
const Statement = require("./statement");
const Event = require("./event");
const transaction = require("./transaction");
const Macro = require("./macro");
const config = require("./config");
let eventExtension;

try {
  const { path } = config();
  eventExtension = require(`${path}/extensions/event.js`);
} catch (err) {} // eslint-disable-line no-empty

module.exports.process = function (statement, options = {}) {
  options = { ...config(), ...options };
  const { declarative, details, cacheOnly } = options;

  const before = Date.now();
  let statements, result, error, execs;

  let s = Macro.apply(statement, options);

  try {
    statements = Statement.compile(s, options);
    transaction.start();
    result = stack.process(statements, null, options);
    const list = transaction.end();
    execs = list.filter((item) => item.exec).map((item) => item.exec);
  } catch (err) {
    transaction.rollback();
    result = err;
    error = true;
  }

  const events = Event.list();
  const date = Date.now();
  const time = date - before;

  if (!cacheOnly) {
    if (result instanceof Error) {
      result = `${result.constructor.name}: ${result.message}`;
    }

    datastore.write({
      s,
      c: declarative ? true : undefined,
      t: time,
      r: result,
      d: date,
      e: error,
      v: events,
      x: execs,
    });
  }

  if (eventExtension) {
    // TODO Disable for restart
    eventExtension.apply(events);
  }

  Event.clear();

  if (details) {
    if (result instanceof Error) {
      result = `${result.constructor.name}: ${result.message}`;
    }

    return {
      result,
      date,
      time,
      error,
      execs,
      events,
    };
  } else {
    if (error) {
      throw result;
    }

    return result;
  }
};
