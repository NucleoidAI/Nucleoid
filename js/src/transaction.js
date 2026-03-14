let list = [];

function start() {
  list = [];
}

function end() {
  const result = list;
  list = [];
  return result;
}

function register(p1, p2, p3) {
  // eslint-disable-next-line no-unused-vars
  const state = require("./state").$;

  if (!p1) return;

  if (typeof p1 === "string") {
    const variable = p1;
    const value = p2;

    // eslint-disable-next-line no-eval
    const before = eval(`${variable}`);

    // eslint-disable-next-line no-eval
    const after = eval(`${variable}=${value}`);

    list.push({ variable, before });
    return after;
  } else {
    let object = p1;
    let property = p2;
    let value = p3;

    list.push({ object, property, before: object[property] });
    object[property] = value;
  }
}

function rollback() {
  const state = require("./state").$; // eslint-disable-line no-unused-vars

  while (list.length) {
    let transaction = list.pop();
    let variable = transaction.variable;
    let object = transaction.object;
    let property = transaction.property;
    let before = transaction.before;

    if (variable !== undefined) {
      // eslint-disable-next-line no-eval
      eval(`${variable}=before`);
    } else {
      if (before === undefined) {
        delete object[property];
      } else {
        object[property] = before;
      }
    }
  }
}

module.exports.start = start;
module.exports.end = end;
module.exports.register = register;
module.exports.rollback = rollback;
