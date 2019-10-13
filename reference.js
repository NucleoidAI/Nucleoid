var state = require("./state"); // eslint-disable-line no-unused-vars
var Value = require("./value");

module.exports = class REFERENCE extends Value {
  run() {
    return eval("state." + this.link);
  }

  graph() {
    return [this.link];
  }
};
