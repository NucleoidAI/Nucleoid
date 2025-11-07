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

export { $, retrieve, clear };
