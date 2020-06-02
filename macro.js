var state = require("./state");
var funcRegex = /[A-z0-9 $_.]+\(.+\)/;

module.exports.apply = function(string) {
  return string.replace(funcRegex, function(fn) {
    try {
      let fnName = fn.substr(0, fn.indexOf("("));
      let value = state.run(null, `${fnName}.value`);

      if (value === true) {
        return state.run(null, fn);
      } else {
        return fn;
      }
    } catch (e) {
      return fn;
    }
  });
};

Date.now.value = true;
Date.UTC.value = true;
