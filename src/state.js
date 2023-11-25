/* eslint-disable no-unused-vars */
/* eslint-disable no-eval */
// TODO Rename this to `$`
const state = {
  classes: [],
};
const _transaction = require("./transaction");
const { graph: _graph } = require("./graph");
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
function assign(scope, variable, evaluation, json = true) {
  let value;

  if (json) {
    value = serialize(eval(`(${evaluation})`), "state");
  } else {
    value = evaluation.toString();
  }

  return _transaction.register(`state.${variable}`, value);
}

function call(scope, fn, args = []) {
  const exec = `state.${fn}(${args.join(",")})`;
  return eval(exec);
}

function expression(scope, evaluation) {
  return eval(`(${evaluation.value})`);
}

function load(execs = []) {
  const Node = require("./nuc/NODE");
  const graph = require("./graph");

  execs.forEach((exec) => {
    eval(exec);
  });
}

module.exports.throw = (scope, exception) => eval(`throw ${exception}`);

function clear() {
  for (let property in state) {
    delete state[property];
  }

  state["classes"] = [];
}

module.exports.state = state; // will be private
module.exports.assign = assign;
module.exports.call = call;
module.exports.expression = expression;
module.exports.load = load;
module.exports.clear = clear;
