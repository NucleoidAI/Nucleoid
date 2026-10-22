import { inspect, stripVTControlCharacters, styleText } from "node:util";

const palette = [
  [0, 0, 0],
  [0, 116, 217],
  [255, 65, 54],
  [46, 204, 64],
  [255, 220, 0],
  [170, 170, 170],
  [240, 18, 190],
  [255, 133, 27],
  [127, 219, 255],
  [135, 12, 37],
];

function width() {
  return Math.max(8, Math.min(88, (process.stdout.columns ?? 92) - 4));
}

function colors() {
  return styleText("red", ".") !== ".";
}

function wrap(text, columns) {
  const lines = [""];
  for (const word of text.split(/\s+/)) {
    const index = lines.length - 1;
    if (lines[index] && lines[index].length + word.length + 1 > columns) {
      lines.push(word);
    } else {
      lines[index] += `${lines[index] ? " " : ""}${word}`;
    }
  }
  return lines;
}

function block(title, lines, write = console.log) {
  const [label, ...continuation] = wrap(
    stripVTControlCharacters(String(title)),
    width() - 4
  );
  const rule = "─".repeat(Math.max(0, width() - Array.from(label).length - 4));
  write(
    `\n  ${styleText("gray", "╭─")} ${styleText("bold", label)} ${styleText(
      "gray",
      rule
    )}`
  );
  for (const line of continuation) {
    write(`  ${styleText("gray", "│")}  ${styleText("bold", line)}`);
  }
  for (const line of lines) {
    write(`  ${styleText("gray", "│")}  ${line}`);
  }
  write(`  ${styleText("gray", `╰${"─".repeat(width() - 1)}`)}`);
}

export function header({ trainingPairs, columns, rows }) {
  if (width() < 50) {
    console.log(`\n  ${styleText(["bold", "yellow"], "✦ NUCLEOID ARC")}`);
    console.log(`  Training  ${trainingPairs} pairs`);
    console.log(`  Input     ${columns} x ${rows}\n`);
    return;
  }

  const lines = [
    "",
    styleText(["bold", "yellow"], "   ✦  NUCLEOID ARC"),
    styleText("gray", "      Inspired by Nature."),
    "",
    "      Analyze  /  Construct  /  Visualize",
    "",
    `      Training  ${trainingPairs} pairs     Input  ${columns} x ${rows}`,
    "",
  ];
  console.log(`\n  ${styleText("gray", `╭${"─".repeat(width() - 2)}╮`)}`);
  for (const line of lines) {
    const padding = Math.max(
      0,
      width() - 2 - Array.from(stripVTControlCharacters(line)).length
    );
    console.log(
      `  ${styleText("gray", "│")}${line}${" ".repeat(padding)}${styleText(
        "gray",
        "│"
      )}`
    );
  }
  console.log(`  ${styleText("gray", `╰${"─".repeat(width() - 2)}╯`)}\n`);
}

export function section(index, title, detail) {
  const label = `${String(index).padStart(2, "0")}  ${title.toUpperCase()}`;
  const rule = "─".repeat(Math.max(0, width() - label.length - 4));
  console.log(
    `\n  ${styleText("yellow", "◆")} ${styleText("bold", label)} ${styleText(
      "gray",
      rule
    )}`
  );
  for (const line of wrap(detail, width() - 2)) {
    console.log(`    ${styleText("gray", line)}`);
  }
  console.log("");
}

export function step(message) {
  console.log(`  ${styleText("yellow", "›")} ${message}`);
}

export function code(title, source) {
  const text = Array.isArray(source) ? source.join("\n\n") : String(source);
  block(
    title,
    stripVTControlCharacters(text)
      .split("\n")
      .map((line) => styleText("cyan", line))
  );
}

export function value(title, data) {
  block(
    title,
    inspect(data, {
      colors: colors(),
      depth: null,
      compact: false,
      breakLength: Math.max(8, width() - 6),
      maxArrayLength: null,
      maxStringLength: null,
    }).split("\n")
  );
}

function cell(value, colored) {
  const text = `${String(value).padStart(2)} `;
  if (!colored || !Number.isInteger(value) || value < 0 || value > 9) {
    return text;
  }
  const [red, green, blue] = palette[value];
  const foreground = red * 0.299 + green * 0.587 + blue * 0.114 > 160 ? 30 : 97;
  return `\x1b[48;2;${red};${green};${blue}m\x1b[${foreground}m${text}\x1b[0m`;
}

export function grid(title, matrix) {
  const rows = matrix.length;
  const columns = Math.max(0, ...matrix.map((row) => row.length));
  const dimensions = `${columns} × ${rows}`;
  if (rows === 0 || columns === 0) {
    block(`${title} · ${dimensions}`, [styleText("gray", "(empty)")]);
    return;
  }

  const rowWidth = String(rows - 1).length;
  const chunk = Math.max(1, Math.floor((width() - rowWidth - 6) / 3));
  const colored = colors();
  for (let start = 0; start < columns; start += chunk) {
    const end = Math.min(columns, start + chunk);
    const axis =
      " ".repeat(rowWidth + 2) +
      Array.from({ length: end - start }, (_, index) =>
        `${start + index}`.padStart(2).concat(" ")
      ).join("");
    block(`${title} · ${dimensions}`, [
      ...(columns > chunk
        ? [styleText("gray", `Columns ${start}–${end - 1}`), ""]
        : []),
      styleText("gray", axis),
      ...matrix.map(
        (row, index) =>
          styleText("gray", `${String(index).padStart(rowWidth)}  `) +
          row
            .slice(start, end)
            .map((value) => cell(value, colored))
            .join("")
      ),
    ]);
  }
}

export function usage({ prompt_tokens, completion_tokens }) {
  console.log(
    styleText(
      "gray",
      `    ↳ Model usage  ${prompt_tokens} input / ${completion_tokens} output tokens`
    )
  );
}

export function summary(difference, duration) {
  const seconds = duration / 1000;
  const elapsed =
    seconds < 60
      ? `${seconds.toFixed(1)}s`
      : `${Math.floor(seconds / 60)}m ${(seconds % 60).toFixed(1)}s`;
  const status =
    difference === 0
      ? styleText(["bold", "green"], "✓ EXACT MATCH")
      : styleText(["bold", "yellow"], "◇ MISMATCH");
  block("Run complete", [
    status,
    "",
    `Difference  ${difference}`,
    `Elapsed     ${elapsed}`,
  ]);
  console.log("");
}

export function failure(error) {
  const text = inspect(error, {
    colors: false,
    depth: null,
    maxArrayLength: null,
    maxStringLength: null,
  });
  block(
    "Run failed",
    stripVTControlCharacters(text)
      .split("\n")
      .map((line) => styleText("red", line)),
    console.error
  );
}
