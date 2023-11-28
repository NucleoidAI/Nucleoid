const _ = require("lodash");

function root(node) {
  let current = node;

  while (
    !["Identifier", "ThisExpression", "Literal"].includes(current.object?.type)
  ) {
    current = current.object;
  }

  return current;
}

function append(source, target) {
  const clonedSource = _.cloneDeep(source);
  const clonedTarget = _.cloneDeep(target);

  if (!source) {
    return clonedTarget;
  }

  if (!target) {
    return clonedSource;
  }

  const empty = {
    type: "MemberExpression",
    computed: false,
  };

  if (clonedTarget.type === "Identifier") {
    empty.property = clonedTarget;
    empty.object = clonedSource;

    return empty;
  } else if (clonedTarget.type === "MemberExpression") {
    const first = root(clonedTarget);

    empty.property = first.object;
    empty.object = clonedSource;
    first.object = empty;

    return clonedTarget;
  } else {
    throw new Error("Invalid expression type");
  }
}

module.exports = { append, root };
