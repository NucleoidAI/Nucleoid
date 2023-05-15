const REFERENCE = require("./nuc/REFERENCE");
const serialize = require("./lib/serialize");

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
  const { state } = require("./state");

  if (!p1) return;

  if (typeof p1 === "string") {
    let variable = p1;
    let expression = p2;
    let scope = p3; // eslint-disable-line no-unused-vars

    let transaction;

    if (expression instanceof REFERENCE) {
      // eslint-disable-next-line no-eval
      const before = eval(`state.${variable}`);
      const exec = `state.${variable}=${expression.run()}`;

      transaction = { variable, exec, before };
    } else {
      const before = eval(`state.${variable}`); // eslint-disable-line no-eval
      const result = eval(`(${expression})`); // eslint-disable-line no-eval
      const value = serialize(result);
      const exec = `state.${variable}=${value}`;

      transaction = { variable, exec, before };
    }

    list.push(transaction);
    // eslint-disable-next-line no-eval
    return eval(transaction.exec);
  } else {
    let object = p1;
    let property = p2;
    let value = p3;

    list.push({ object, property, before: object[property] });
    object[property] = value;
  }
}

function push(exec) {
  list.push({ exec });
}

function rollback() {
  const { state } = require("./state"); // eslint-disable-line no-unused-vars

  while (list.length) {
    let transaction = list.pop();
    let variable = transaction.variable;
    let object = transaction.object;
    let property = transaction.property;
    let before = transaction.before;

    if (!variable && !object && !property) {
      continue;
    }

    if (variable !== undefined) {
      // eslint-disable-next-line no-eval
      eval(`state.${variable}=before`);
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
module.exports.push = push;
module.exports.register = register;
module.exports.rollback = rollback;
