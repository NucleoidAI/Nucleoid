class $ {
  constructor() {
    this.iof = this.constructor.name;
    this.pre = false;
  }

  before() {}
  run() {}
  graph() {}
  after() {}

  get prepared() {
    return this.pre;
  }

  set prepared(prepared) {
    this.pre = prepared;
  }
}

module.exports = $;
