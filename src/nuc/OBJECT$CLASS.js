const Node = require("./NODE");
const Id = require("../lib/identifier");
const OBJECT$INSTANCE = require("./OBJECT$INSTANCE");

class OBJECT$CLASS extends Node {
  before() {
    this.key = Id.serialize(this);
  }

  run(scope) {
    let instances;
    let statements = [];

    let instance = scope.instance(Id.root(this).name);

    if (instance) instances = [instance];
    else instances = Object.values(Id.root(this).instances);

    for (let instance of instances) {
      let statement = new OBJECT$INSTANCE();
      statement.instance = instance;
      statement.name = this.name;
      statement.declaration = this;
      statements.push(statement);
    }

    return { next: statements };
  }

  graph() {
    Id.root(this).declarations[this.key] = this;
  }
}

OBJECT$CLASS.prototype.type = "CLASS";
module.exports = OBJECT$CLASS;
