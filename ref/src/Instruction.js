class Instruction {
  constructor(
    scope,
    statement,
    before,
    run,
    graph,
    after,
    derivative = true,
    priority = false
  ) {
    this.scope = scope;
    this.statement = statement;
    this.before = before;
    this.run = run;
    this.graph = graph;
    this.after = after;
    this.derivative = derivative;
    this.priority = priority;
  }
}

module.exports = Instruction;
