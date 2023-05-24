const Id = require("../lib/identifier");
const EXPRESSION = require("./EXPRESSION");
const Local = require("../lib/local");
const Token = require("../lib/token");

class REFERENCE extends EXPRESSION {
  before() {}

  run(scope) {
    if (this.link.name) {
      let local = Local.retrieve(scope, this.link.name);

      if (local) return local;
    }

    let arr = new Token.ARRAY();
    arr.push(new Token("state." + Id.serialize(this.link)));
    return arr;
  }

  next() {}

  graph() {
    return [Id.serialize(this.link)];
  }
}

module.exports = REFERENCE;
