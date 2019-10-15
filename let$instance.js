var LET = require("./let");
var EXPRESSION = require("./expression");
var Identifier = require("./identifier");

class LET$INSTANCE extends LET {
  run(scope) {
    this.value = new EXPRESSION(
      this.declaration.value.tokens.map(token => {
        let parts = token.split(".");
        if (parts[0] == this.class.name)
          parts[0] = Identifier.serialize(this.instance, true);
        return parts.join(".");
      })
    );

    let expression = this.value.run(scope, this.instance); // eslint-disable-line no-unused-vars
    eval("scope.local." + Identifier.serialize(this, false) + "=expression");
  }

  // eslint-disable-next-line no-unused-vars
  graph(scope) {
    eval("scope.graph." + Identifier.serialize(this, false) + "=this");
    return this.value.graph();
  }
}

LET$INSTANCE.prototype.type = "INSTANCE";
module.exports = LET$INSTANCE;
