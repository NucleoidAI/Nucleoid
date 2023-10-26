class Expression {
  constructor(node) {
    this.node = node;
  }

  traverseReduce(exp, fn, acc = []) {
    if (exp.type === "BinaryExpression") {
      this.traverseReduce(exp.left, fn, acc);
      acc.push(exp.operator);
      this.traverseReduce(exp.right, fn, acc);
    } else {
      const cur = fn(exp);

      if (cur) {
        acc.push(cur);
      }
    }

    return acc;
  }

  filterReduce(exp, fn, acc = []) {
    if (exp.type === "BinaryExpression") {
      this.filterReduce(exp.left, fn, acc);
      this.filterReduce(exp.right, fn, acc);
    } else {
      const cur = fn(exp);

      if (cur) {
        acc.push(cur);
      }
    }

    return acc;
  }

  filter(fn) {
    return this.filterReduce(this.node, fn);
  }

  traverse(fn) {
    return this.traverseReduce(this.node, fn);
  }
}

module.exports = Expression;
