const Node = require("./node");

module.exports = class FUNCTION extends Node {
  before() {
    this.key = this.name;
  }
};
