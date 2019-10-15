module.exports = class Scope {
  constructor(prior, block) {
    this.prior = prior;
    this.block = block;

    if (prior) {
      this.root = prior.root;
    } else {
      this.root = this;
    }

    this.local = {};
    this.instance = {};
    this.graph = {};
  }

  retrieve(instance) {
    let index = this;

    while (index) {
      let value = index.instance[instance];
      if (value) return value;
      index = index.prior;
    }
  }
};
