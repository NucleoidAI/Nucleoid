const Node = require("./Node");
const Id = require("../lib/identifier");
const graph = require("../graph");
const OBJECT$INSTANCE = require("./OBJECT$INSTANCE");
const Instance = require("../lib/instance");

class OBJECT$CLASS extends Node {
  before() {
    this.key = Id.serialize(this);
  }

  run(scope) {
    let instances;
    let statements = [];

    let instance = Instance.retrieve(scope, Id.root(this).name);

    if (instance) instances = [instance];
    else instances = Object.keys(Id.root(this).instances).map((i) => graph[i]);

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
