import { readFileSync } from "node:fs";
import { join } from "node:path";
import { v4 as uuid } from "uuid";
import * as dataset from "../dataset";
import * as openrouter from "./openrouter";

const init_class: string = [
  "# Each object is one group of cells on the grid, located by its bounding box",
  "class Obj(x: int, y: int, width: int, height: int, orientation: int, volume: int):",
  "    this.x is x",
  "    this.y is y",
  "    this.width is width",
  "    this.height is height",
  "    this.orientation is orientation",
  "    this.volume is volume",
].join("\n");

export async function generate({
  puzzle_id = "3aa6fb7a",
  only_puzzle_dataset = false,
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
      content: JSON.stringify({
        init_class: { class_declaration: init_class },
      }),
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
