const runtime = require("./runtime");

module.exports.run = function (statement, options) {
  return runtime.process(statement, options);
};
