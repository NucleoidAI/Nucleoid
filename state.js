var state = {}; // eslint-disable-line no-unused-vars
var transaction = require("./transaction");
var message = require("./message").message; // eslint-disable-line no-unused-vars

module.exports.state = state;
module.exports.assign = function(scope, variable, expression) {
  transaction.register(variable, expression, scope);
};

module.exports.run = function(scope, expression) {
  // eslint-disable-next-line no-eval
  return eval(expression);
};
