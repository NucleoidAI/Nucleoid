class Evaluation {
  constructor(value, transactions) {
    this.value = value;
    this.transactions = transactions;
  }

  toString() {
    return this.value;
  }
}

module.exports = Evaluation;
