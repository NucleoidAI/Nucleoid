var ControlFlow = require("./controlflow");

module.exports.run = function(string) {
  let controlFlow = new ControlFlow(string);

  controlFlow.configure(function(statement) {
    switch (statement.type) {
      case "VAR":
        return statement.run();

      default:
        return statement.run();
    }
  });

  return controlFlow.run();
};
