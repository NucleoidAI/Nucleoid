import * as escodegen from "escodegen";

import { FORMAT_MINIFY } from "escodegen";

interface Node {
  type: string;
  [key: string]: any;
}

function generate(node: Node): string {
  return escodegen.generate(node, { format: FORMAT_MINIFY });
}

export { generate };
