var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./graph");
var Node = require("./node");
var PROPERTY = require("./property");
var EXPRESSION = require("./expression");
var Identifier = require("./identifier");

class PROPERTY$INSTANCE extends PROPERTY {
  prepare() {
    let prefix = Identifier.serialize(this.object);
    this.id = prefix + "." + this.name;

    this.value = new EXPRESSION(
      this.declaration.value.tokens.map(token => {
        let parts = token.split(".");
        if (parts[0] == this.class.name)
          parts[0] = Identifier.serialize(this.instance);
        return parts.join(".");
      })
    );
  }

  run(scope) {
    let instance = eval("state." + Identifier.serialize(this.object));
    instance[this.name] = this.value.run(scope);
  }

  graph(scope) {
    return this.value.graph(scope).filter(token => {
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
