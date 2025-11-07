type ContextItem = { definition: string; options: object };
let _context: ContextItem[] = [];

function load(context: ContextItem[]): void {
  _context = _context.concat(context);
}

function run(context: ContextItem[] = []): void {
  // eslint-disable-next-line @typescript-eslint/no-var-requires
  const nucleoid = require("../index");

  load(context);

  _context.forEach(({ definition, options }) =>
    nucleoid.run(definition, options)
  );

  _context = [];
}

export { load, run };
