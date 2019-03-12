module.exports = class Node {
  constructor(statement) {
    this.statement = statement;
    this.edge = {};
  }
};
