module.exports.splitLast = function(name) {
  let parts = name.split(".");
  let instance = "";

  for (let i = 0; i < parts.length - 1; i++) {
    instance += parts[i] + ".";
  }

  return [parts[parts.length - 1], instance.slice(0, -1)];
};
