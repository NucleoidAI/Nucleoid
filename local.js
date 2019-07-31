module.exports.retrieve = function(scope, variable) {
  let index = scope;

  while (index) {
    let value = index.local[variable];
    if (value) return index.local[variable];
    index = index.prior;
  }
};
