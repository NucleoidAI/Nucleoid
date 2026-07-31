// eslint-disable-next-line no-unused-vars
const state = require("./state").$;
const { append } = require("./lang/estree/estree");
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
    this.instance = null;
    this.graph = {};
    this.callback = [];
  }

  get $instance() {
    let index = this;

    while (index) {
      const instance = index.instance;

      if (instance) {
        return instance;
      }

      index = index.prior;
    }

    return null;
  }

  set $instance(instance) {
    this.instance = instance;
  }

  assign(variable, evaluation, reassign = false) {
    let prefix;

    if (reassign) {
      prefix = this.retrieve(variable.first)?.object.node;
    }

    if (!prefix) {
      prefix = {
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
    }

    const local = new Identifier(append(prefix, variable.node));

    // eslint-disable-next-line no-unused-vars
    const scope = this;
    // eslint-disable-next-line no-eval
    return eval(`${local}=${evaluation}`);
  }

  retrieve(variable, exact = false) {
    let index = this;

    let estree = {
      type: "Identifier",
      name: "scope",
    };

    const first = variable.first;

    while (index) {
      if (
        index.graph[first] !== undefined &&
        // eslint-disable-next-line no-eval
        (!exact || eval(`index.local.${variable}`) !== undefined)
      ) {
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

  retrieveObject() {
    let index = this;

    while (index) {
      if (index.object !== undefined) {
        return index.object.name;
      }

      index = index.prior;
    }

    return null;
  }

  retrieveGraph(instance) {
    let index = this;

    while (index) {
      const value = index.graph[instance];

      if (value) {
        return value;
      }

      index = index.prior;
    }
  }
}

module.exports = Scope;
