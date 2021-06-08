const state = require("./state");
const funcRegex = /[A-z0-9$_.]+[ ]*\(.*?\)/g;

module.exports.apply = function (string) {
  return string.replace(funcRegex, function (fn) {
    let fnName = fn.substr(0, fn.indexOf("("));

    try {
      let value = state.run(null, `state.${fnName}.value`);

      if (value === true) {
        return JSON.stringify(state.run(null, `state.${fn}`));
      } else {
        return fn;
      }
    } catch (e) {
      // continue regardless of error
    }

    try {
      let value = state.run(null, `${fnName}.value`);

      if (value === true) {
        return JSON.stringify(state.run(null, fn));
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
