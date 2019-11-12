module.exports.retrieve = function(scope, instance) {
  let index = scope;

  while (index) {
    let value = index.instance[instance];
    if (value) return value;
    index = index.prior;
  }
};
