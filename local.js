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
      if (index.graph[assignment] !== undefined) return true;
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
    let node = scope.graph[reference];

    if (node && node.value && node.value.instanceof === "REFERENCE") {
      parts = parts.slice(i, parts.length);
      parts[0] = node.value.link.key;
      i = 0;
    }
  }

  return parts.join(".");
};

module.exports.object = function(scope) {
  let index = scope;

  while (index) {
    if (index.object !== undefined) {
      return index.object.name;
    }

    index = index.prior;
  }
};
