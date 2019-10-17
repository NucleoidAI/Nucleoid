var graph = require("./graph");
var REFERENCE = require("./reference");

module.exports.splitLast = function(name) {
  let parts = name.split(".");
  let instance = "";

  for (let i = 0; i < parts.length - 1; i++) {
    instance += parts[i] + ".";
  }

  return [parts[parts.length - 1], instance.slice(0, -1)];
};

module.exports.serialize = function(node, reference) {
  let string = "";
  let index = node;

  while (index) {
    if (reference && index.value instanceof REFERENCE) {
      index = index.value.link;
    }

    string = index.name + "." + string;
    index = index.object;
  }

  return string.slice(0, -1);
};

module.exports.reference = function(name) {
  let parts = name.split(".");

  for (let i = 0; i < parts.length; i++) {
    let reference = parts.slice(0, i + 1).join(".");

    if (graph[reference] && graph[reference].value instanceof REFERENCE) {
      parts = parts.slice(i, parts.length);
      parts[0] = graph[reference].value.link.key;
      i = 0;
    }
  }

  return parts.join(".");
};
