var state = { variables: [] }; // eslint-disable-line no-unused-vars

module.exports.run = function(statement) {
  statement = statement
    .trim()
    .replace(/^(var\s+)|([A-z]\w*(?=\s*=(?!=)))/g, function(match, pattern) {
      if (pattern) {
        return "";
      } else {
        state.variables[match] = true;
        return match;
      }
    })
    .replace(/[A-z]\w*/g, function(match) {
      if (state.variables[match]) {
        return "state.".concat(match);
      } else {
        return match;
      }
    });

  return eval(statement);
};
