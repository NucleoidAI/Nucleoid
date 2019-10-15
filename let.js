class LET {
  run(scope) {
    let value = this.value.run(scope); // eslint-disable-line no-unused-vars
    eval("scope.local." + this.name + "=value");
  }

  graph() {
    return this.value.graph();
  }
}

LET.prototype.type = "REGULAR";
module.exports = LET;
