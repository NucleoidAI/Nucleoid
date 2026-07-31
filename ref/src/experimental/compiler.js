const ts = require("typescript");

function compile(string) {
  const sourceFile = ts.createSourceFile(
    "temp.ts",
    string,
    ts.ScriptTarget.ES2015,
    false,
    ts.ScriptKind.TS
  );

  // TODO It should returns AST of string
  return sourceFile;
}

module.exports.compile = compile;
