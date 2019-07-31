module.exports = class Scope {
  constructor(prior) {
    this.prior = prior;

    if (prior) {
      this.root = prior.root;
    } else {
      this.root = this;
    }

    this.local = {};
    this.instance = {};
  }

  retrieve(scope, instance) {
    let index = scope;

    while (index) {
      let value = index.instance[instance];
      if (value) return value;
      index = index.prior;
    }
  }
};
