var state = {}; // eslint-disable-line no-unused-vars

module.exports.run = function(statement) {
  statement = statement
    .trim()
    .replace(/^(var\s+)/, "")
    .replace(/[A-z]\w*/g, function(match) {
      return "state.".concat(match);
    });

  return eval(statement);
};
