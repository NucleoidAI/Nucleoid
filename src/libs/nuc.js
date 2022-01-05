const runtime = require("../runtime");

module.exports.load = (nuc) => {
  if (!nuc || !nuc.functions) {
    throw Error("Invalid NUC file");
  }

  const { functions } = nuc;
  Object.values(functions).forEach((fn) => {
    try {
      runtime.process(fn.code, { declarative: false });
    } catch (error) {
      console.info("Problem occurred while loading NUC file", error);
    }
  });
};
