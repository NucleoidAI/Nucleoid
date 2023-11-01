// eslint-disable-next-line no-unused-vars
const state = require("./state").state;

class Scope {
  constructor(prior, block) {
    this.prior = prior;
    this.block = block;

    if (prior) {
      this.root = prior.root;
    } else {
      this.root = this;
    }

    this.local = {};
    this.instances = {};
    this.graph = {};
  }

  assign(variable, evaluation) {
    // eslint-disable-next-line no-unused-vars
    const scope = this;

    let local = scope.retrieve(variable);

    if (!local) {
      local = `scope.local.${variable}`;
    }

    // eslint-disable-next-line no-eval
    eval(`${local}=${evaluation.value}`);
  }

  retrieve(variable) {
    let index = this;

    let parts = variable.split(".");
    let reference = "scope.";
    const first = parts[0];

    while (index) {
      if (index.local[first] !== undefined) {
        return reference + "local." + parts.join(".");
      }

      reference += "prior.";
      index = index.prior;
    }
  }
}

module.exports = Scope;
