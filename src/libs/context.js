let nucleoid;

setImmediate(() => {
  nucleoid = require("../../index");
});

let _context = [];

const load = (context) => {
  _context = _context.concat(context);
};

const run = (context = []) => {
  load(context);

  _context.forEach(({ definition, options }) => {
    try {
      nucleoid.run(definition, options);
    } catch (error) {
      console.info("Problem occurred while loading functions", error);
    }
  });
};

module.exports = { load, run };
