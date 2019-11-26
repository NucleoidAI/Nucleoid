var REFERENCE = require("./reference");

module.exports.retrieve = function(scope, assignment) {
  let index = scope;

  let parts = assignment.split(".");
  let reference = "scope.";

  while (index) {
    if (index.local[parts[0]] !== undefined) {
      return reference + "local." + parts.join(".");
    }

    reference += "prior.";
    index = index.prior;
  }
};

module.exports.check = function(scope, assignment) {
  let index = scope;

  if (assignment === undefined) return false;
  try {
    while (index) {
      if (eval("index.local." + assignment) !== undefined) return true;
      index = index.prior;
    }

    return false;
  } catch (error) {
    if (error instanceof TypeError) return false;
    throw error;
  }
};

module.exports.reference = function(scope, name) {
  let parts = name.split(".");

  for (let i = 0; i < parts.length; i++) {
    let reference = parts.slice(0, i + 1).join(".");

    if (
      scope.graph[reference] &&
      scope.graph[reference].value instanceof REFERENCE
    ) {
      parts = parts.slice(i, parts.length);
      parts[0] = scope.graph[reference].value.link.key;
      i = 0;
    }
  }

  return parts.join(".");
};
