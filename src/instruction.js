class Instruction {
  constructor(scope, statement, before, run, graph, root, derivative) {
    this.scope = scope;
    this.statement = statement;
    this.before = before;
    this.run = run;
    this.graph = graph;
    this.root = root;
    this.derivative = derivative;
  }
}

module.exports = Instruction;
