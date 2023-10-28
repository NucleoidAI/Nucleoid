const Identifier = require("./Identifier");
const graph = require("../../graph");

class Call {
  constructor(token) {
    this.token = token;
    this.name = new Identifier(this.token.callee);
  }

  resolve(path = false) {
    const name = this.name.object.resolve();

    if (path) {
      const resolved = graph[name] ? `state.${name}` : name;
      return `${resolved}.${this.name.last.resolve()}()`;
    } else {
      return `${name}.${this.name.last.resolve()}()`;
    }
  }
}

module.exports = Call;
