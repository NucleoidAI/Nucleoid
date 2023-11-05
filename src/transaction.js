let list = [];

function start() {
  list = [];
}

function end() {
  const result = list;
  list = [];
  return result;
}

function assignGraph(object, property, value) {
  if (!object) return;

  list.push({ object, property, before: object[property] });
  object[property] = value;
}

function push(transaction) {
  list.push(transaction);
}

function rollback() {
  // eslint-disable-next-line no-unused-vars
  const { state } = require("./state");

  // TODO Temporary disable transaction rollback
  return;

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
module.exports.assignGraph = assignGraph;
module.exports.rollback = rollback;
