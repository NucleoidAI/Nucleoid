// eslint-disable-next-line no-unused-vars
const state = {
  classes: [],
};
const _transaction = require("./transaction");
const _graph = require("./graph");
const event = require("./event").event; // eslint-disable-line no-unused-vars
const _ = require("lodash"); // eslint-disable-line no-unused-vars

global.require = require;

// eslint-disable-next-line no-unused-vars
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
function assign(scope, variable, expression, adjust) {
  return _transaction.register(variable, expression, scope, adjust);
}

function run(scope, expression) {
  // eslint-disable-next-line no-eval
  return eval(`(${expression})`);
}

module.exports.state = state;
module.exports.assign = assign;
module.exports.run = run;
// eslint-disable-next-line no-eval
module.exports.throw = (scope, exception) => eval(`throw state.${exception}`);
