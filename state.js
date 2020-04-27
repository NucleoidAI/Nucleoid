var state = {}; // eslint-disable-line no-unused-vars
var message = require("./message").message; // eslint-disable-line no-unused-vars

module.exports.state = state;
module.exports.assign = function(scope, variable, expression) {
  if (typeof expression === "string") {
    // eslint-disable-next-line no-eval
    eval(`state.${variable}=${expression}`);
  } else {
    // eslint-disable-next-line no-eval
    eval(`state.${variable}=expression`);
  }
};

module.exports.run = function(scope, expression) {
  // eslint-disable-next-line no-eval
  return eval(expression);
};
