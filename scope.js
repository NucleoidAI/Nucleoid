module.exports = class Scope {
  constructor(type) {
    this.type = type;
    this.local = {};
  }
};
