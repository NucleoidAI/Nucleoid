var $CLASS = require("./$class").$CLASS;
var LET$INSTANCE = require("./let$instance");

module.exports = class LET$CLASS extends $CLASS {
  run() {
    if (this.instance) {
      let instance = new LET$INSTANCE();
      instance.class = this.class;
      instance.instance = this.instance;
      instance.variable = this.variable;
      instance.expression = this.expression;
      return instance;
    }
  }

  graph() {}
};
