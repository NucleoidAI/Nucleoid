class Literal {
  constructor(token) {
    this.token = token;
  }

  resolve() {
    return this.token.raw;
  }
}

module.exports = Literal;
