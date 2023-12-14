const graph = require("../graph");

function splitLast(name) {
  let parts = name.split(".");
  let instance = "";

  if (parts.length === 1) {
    return [name];
  }

  for (let i = 0; i < parts.length - 1; i++) {
    instance += parts[i] + ".";
  }

  return [parts[parts.length - 1], instance.slice(0, -1)];
}

function serialize(node, reference) {
  let string = "";
  let index = node;

  while (index) {
    if (reference && index.value && index.value.instanceof === "REFERENCE") {
      index = graph.retrieve(index.value.link);
    }

    string = index.name + "." + string;
    index = index.object;
  }

  return string.slice(0, -1);
}

function reference(name) {
  let parts = name.split(".");

  for (let i = 0; i < parts.length; i++) {
    let reference = parts.slice(0, i + 1).join(".");
    let node = graph[reference];

    if (node?.value?.instanceof === "REFERENCE") {
      parts = parts.slice(i, parts.length);
      parts[0] = graph[reference].value.link.key;
      i = 0;
    }
  }

  return parts.join(".");
}

function root(node) {
  let index = node;

  while (index.object) {
    index = index.object;
  }

  return index;
}

module.exports.splitLast = splitLast;
module.exports.serialize = serialize;
module.exports.reference = reference;
module.exports.root = root;
