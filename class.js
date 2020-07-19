var state = require("./state");
var Node = require("./node");
var $VAR = require("./$variable");
var $EXP = require("./$expression");
var graph = require("./graph");

var CLASS = class CLASS extends Node {
  constructor() {
    super();
    this.instances = {};
    this.declarations = {};
  }

  before() {
    this.key = this.name;
  }

  run(scope) {
    state.assign(scope, this.name, `class ${this.name}{}`);

    let list = [];

    let classContext = $EXP(`Classes.push(${this.name})`, 0);
    list.push(classContext.statement);

    let instanceContext = $EXP("[]", 0);
    list.push($VAR(this.name + "s", instanceContext.statement));

    return list;
  }

  beforeGraph() {
    if (graph[this.key] && graph[this.key] instanceof CLASS) {
      this.declarations = graph[this.key].declarations;
    }
  }
};

CLASS.prototype.instanceof = "CLASS";
module.exports = CLASS;
