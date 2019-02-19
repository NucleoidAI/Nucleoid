var Statement = require("./statement");

var state = { variables: [] }; // eslint-disable-line no-unused-vars

module.exports.run = function(string) {
  let statement = new Statement(string);

  if (statement.next() == "var") {
    statement.skip();
  }

  if (statement.check() == "=") {
    statement.mark();
    state.variables[statement.token] = true;
  } else {
    if (state.variables[statement.token]) {
      statement.mark();
    }
  }

  statement.scan(token => state.variables[token]);
  return eval(statement.toString());
};
