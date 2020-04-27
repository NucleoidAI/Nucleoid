var state = require("./state");
var Node = require("./node");
var $VAR = require("./$variable");
var $EXP = require("./$expression");
var graph = require("./graph");

var CLASS = class CLASS extends Node {
  constructor() {
    super();
    this.instance = {};
    this.declaration = {};
  }

  before() {
    this.key = this.name;
  }

  run(scope) {
    state.assign(scope, this.name, "class {}");
    let context = $EXP("[]", 0);
    return $VAR(this.name + "s", context.statement);
  }

  beforeGraph() {
    if (graph[this.key] && graph[this.key] instanceof CLASS) {
      this.declaration = graph[this.key].declaration;
    }
  }
};

CLASS.prototype.instanceof = "CLASS";
module.exports = CLASS;
