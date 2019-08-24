module.exports.retrieve = function(scope, assignment) {
  let index = scope;

  let parts = assignment.split(".");
  let reference = "scope.";

  while (index) {
    if (index.local[parts[0]] != undefined) {
      return reference + "local." + parts.join(".");
    }

    reference += "prior.";
    index = index.prior;
  }
};
