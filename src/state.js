/* eslint-disable no-unused-vars */
/* eslint-disable no-eval */
const state = {
  classes: [],
};
const _transaction = require("./transaction");
const _graph = require("./graph");
const { event } = require("./event");
const _ = require("lodash");
const { v4: uuid } = require("uuid");
const REFERENCE = require("./nuc/REFERENCE");
const serialize = require("./lib/serialize");

global.require = require;

function graph(id) {
  let node = _graph[id];

  if (_graph[id] === undefined) {
    throw ReferenceError(`${id} is not defined`);
  }

  let object = {};

  for (let prop in node) {
    let n = node[prop];

    if (typeof n === "object") {
      object[prop] = {};

      for (let nprop in n) {
        if (typeof n[nprop] === "object") {
          object[prop][nprop] = {
            id: n[nprop].id,
            key: n[nprop].key,
            type: n[nprop].constructor.name,
          };
        } else {
          object[prop][nprop] = n[nprop];
        }
      }
    } else {
      object[prop] = node[prop];
    }
  }

  return object;
}
function assign(scope, variable, expression) {
  if (!variable) return;

  let transaction;

  if (expression instanceof REFERENCE) {
    // eslint-disable-next-line no-eval
    const before = eval(`state.${variable}`);
    const run = expression.run();
    const exec = `state.${variable}=${run.construct()}`;

    transaction = { variable, exec, before };
  } else {
    const before = eval(`state.${variable}`); // eslint-disable-line no-eval
    const result = eval(`(${expression})`); // eslint-disable-line no-eval
    const value = serialize(result, "state");
    const exec = `state.${variable}=${value}`;

    transaction = { variable, exec, before };
  }

  _transaction.push(transaction);
  return eval(transaction.exec);
}

function run(scope, expression, transaction = false) {
  if (transaction) {
    _transaction.push({ exec: `(${expression})` });
  }

  return eval(`(${expression})`);
}

function load(execs = []) {
  const Node = require("./nuc/Node");
  const graph = require("./graph");

  execs.forEach((exec) => {
    eval(exec);
  });
}

module.exports.state = state;
module.exports.assign = assign;
module.exports.run = run;
module.exports.throw = (scope, exception) => eval(`throw state.${exception}`);
module.exports.load = load;
