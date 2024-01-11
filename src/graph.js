// TODO Rename this to `$`
const $ = {
  classes: {
    name: "classes",
  },
};

function retrieve(identifier) {
  if (typeof identifier === "string") {
    return $[identifier];
  } else {
    return $[identifier.generate()];
  }
}

function clear() {
  for (let property in $) {
    delete $[property];
  }

  $["classes"] = { name: "classes" };
}

module.exports.$ = $;
module.exports.retrieve = retrieve;
module.exports.clear = clear;
