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
  let statements, result, error;

  statement = `(()=>{${statement}})()`;
  const macro = Macro.apply(statement, options);

  try {
    statements = Statement.parse(macro, options);
    transaction.start();
    result = stack.process(statements, null, options);
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
      s: statement,
      c: declarative ? true : undefined,
      t: time,
      r: result,
      d: date,
      e: error,
      v: events,
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
      events,
    };
  } else {
    if (error) {
      throw result;
    }

    return result;
  }
};
