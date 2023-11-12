// eslint-disable-next-line no-unused-vars
const state = require("./state").state;
const { append } = require("./lang/estree/estree");
const { generate } = require("./lang/estree/generator");
const Identifier = require("./lang/ast/Identifier");

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
    let local = this.retrieve(variable);

    if (!local) {
      const prefix = {
        type: "MemberExpression",
        computed: false, // false because it uses dot notation
        object: {
          type: "Identifier",
          name: "scope",
        },
        property: {
          type: "Identifier",
          name: "local",
        },
      };
      local = append(prefix, variable.node);
    }

    // eslint-disable-next-line no-unused-vars
    const scope = this;
    // eslint-disable-next-line no-eval
    return eval(`${generate(local)}=${evaluation}`);
  }

  retrieve(variable) {
    let index = this;

    let estree = {
      type: "Identifier",
      name: "scope",
    };

    const first = variable.first;

    while (index) {
      if (index.local[first.generate()] !== undefined) {
        const local = {
          type: "Identifier",
          name: "local",
        };
        estree = append(estree, local);
        return new Identifier(append(estree, variable.node));
      }

      const prior = {
        type: "Identifier",
        name: "prior",
      };

      estree = append(estree, prior);

      index = index.prior;
    }

    return null;
  }

  instance(instance) {
    let index = this;

    while (index) {
      const value = index.instances[instance];

      if (value) {
        return value;
      }

      index = index.prior;
    }
  }
}

module.exports = Scope;
