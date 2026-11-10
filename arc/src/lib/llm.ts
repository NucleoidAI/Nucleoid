import { readFileSync } from "node:fs";
import { join } from "node:path";
import { v4 as uuid } from "uuid";
import * as dataset from "../dataset";
import * as openrouter from "./openrouter";

export async function generate({
  puzzle_id,
  only_puzzle_dataset,
  reasoning = false,
  few_shot = (file) => file,
  messages = [],
  grammar = false,
}) {
  if (few_shot) {
    messages.unshift({
      role: "system",
      content: `
        few_shot:
        ${JSON.parse(dataset.document(puzzle_id, only_puzzle_dataset))
          .map(few_shot)
          .map((file) => JSON.stringify(file) + "\n")}
      `,
    });
  }

  if (grammar) {
    messages.unshift({
      role: "system",
      content:
        "init_class:\nclass Obj(x: int, y: int, width: int, height: int, orientation: int):\n   this.x is x\n   this.y is y\n   this.width is width\n   this.height is height\n   this.orientation is orientation\n\n",
    });

    messages.unshift({
      role: "system",
      content: `
        EBNF:
        ${readFileSync(join(__dirname, "..", "..", "nucleoid.ebnf"), "utf8")}
      `,
    });
  }

  messages.unshift({
    role: "system",
    content: `
      control_id: ${uuid()}
    `,
  });

  return openrouter.generate({
    reasoning,
    messages,
  });
}
