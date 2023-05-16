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

// eslint-disable-next-line no-unused-vars
function assign(variable, expression, scope) {
  // eslint-disable-next-line no-unused-vars
  const { state } = require("./state");

  if (!variable) return;

  let transaction;

  if (expression instanceof REFERENCE) {
    // eslint-disable-next-line no-eval
    const before = eval(`state.${variable}`);
    const exec = `state.${variable}=${expression.run()}`;

    transaction = { variable, exec, before };
  } else {
    const before = eval(`state.${variable}`); // eslint-disable-line no-eval
    const result = eval(`(${expression})`); // eslint-disable-line no-eval
    const value = serialize(result, "state");
    const exec = `state.${variable}=${value}`;

    transaction = { variable, exec, before };
  }

  list.push(transaction);
  // eslint-disable-next-line no-eval
  return eval(transaction.exec);
}

function assignGraph(object, property, value) {
  if (!object) return;

  list.push({ object, property, before: object[property] });
  object[property] = value;
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
module.exports.assign = assign;
module.exports.assignGraph = assignGraph;
module.exports.rollback = rollback;
