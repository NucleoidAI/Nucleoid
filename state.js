// eslint-disable-next-line no-unused-vars
const state = {
  Classes: [],
};
const _transaction = require("./transaction");
const _graph = require("./graph");
const message = require("./message").message; // eslint-disable-line no-unused-vars
const event = require("./event").event; // eslint-disable-line no-unused-vars
const OpenAPI = require("./openapi"); // eslint-disable-line no-unused-vars

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

module.exports.state = state;
module.exports.assign = function (scope, variable, expression) {
  _transaction.register(variable, expression, scope);
};

module.exports.run = function (scope, expression) {
  // eslint-disable-next-line no-eval
  return eval(expression);
};
