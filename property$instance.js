var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./graph");
var Node = require("./node");
var PROPERTY = require("./property");
var EXPRESSION = require("./expression");

class PROPERTY$INSTANCE extends PROPERTY {
  run(scope) {
    let prefix = this.instance.identifier();
    this.id = prefix + "." + this.name;

    this.value = new EXPRESSION(
      this.declaration.value.tokens.map(token => {
        let parts = token.split(".");
        if (parts[0] == this.class.name) parts[0] = this.instance.identifier();
        return parts.join(".");
      })
    );

    let instance = eval("state." + prefix);
    instance[this.name] = this.value.run(scope);
  }

  graph() {
    return this.value.graph().filter(token => {
      if (graph.node[token]) return true;
      else if (graph.node[token.split(".")[0]]) {
        graph.node[token] = new Node();
        return true;
      }
    });
  }
}

PROPERTY$INSTANCE.prototype.type = "INSTANCE";
module.exports = PROPERTY$INSTANCE;
