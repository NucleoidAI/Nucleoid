function retrieve(scope, assignment) {
  let index = scope;

  let parts = assignment.split(".");
  let reference = "scope.";
  const first = parts[0];

  while (index) {
    if (index.local[first] !== undefined) {
      return reference + "local." + parts.join(".");
    }

    reference += "prior.";
    index = index.prior;
  }
}

function check(scope, assignment) {
  let index = scope;

  if (assignment === undefined) return false;
  try {
    while (index) {
      if (index.graph[assignment] !== undefined) return index.graph[assignment];
      index = index.prior;
    }
  } catch (error) {
    if (error instanceof TypeError) return false;
    throw error;
  }
}

function reference(scope, name) {
  let parts = name.split(".");

  for (let i = 0; i < parts.length; i++) {
    let reference = parts.slice(0, i + 1).join(".");
    let node = scope.graph[reference];

    if (node?.value?.instanceof === "REFERENCE") {
      parts = parts.slice(i, parts.length);
      parts[0] = node.value.link.key;
      i = 0;
    }
  }

  return parts.join(".");
}

function object(scope) {
  let index = scope;

  while (index) {
    if (index.object !== undefined) {
      return index.object.name;
    }

    index = index.prior;
  }
}

module.exports.retrieve = retrieve;
module.exports.check = check;
module.exports.reference = reference;
module.exports.object = object;
