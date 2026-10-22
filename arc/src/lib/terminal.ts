import { inspect, stripVTControlCharacters, styleText } from "node:util";
import stringWidth from "string-width";
import wrapAnsi from "wrap-ansi";

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

const wordmarks = [
  [
    "⣿⣿⣿⣷⢀⣠⣴⣿⣿⣿⣿⣿⣿⣷⣦⡀      ⢰⣿⣿⣿⣿⡄        ⣾⣿⣿⣿⡆       ⢀⣤⣴⣿⣿⣿⣿⣿⣿⣿⣶⣤⡀",
    "⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣆     ⢸⣿⣿⣿⣿⡇        ⣿⣿⣿⣿⡇      ⣴⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣦⡀",
    "⣿⣿⣿⣿⣿⠟⠉    ⠉⠻⣿⣿⣿⣿⡄    ⢸⣿⣿⣿⣿⡇        ⣿⣿⣿⣿⡇    ⢀⣾⣿⣿⣿⡿⠋     ⠙⢿⣿⣿⣿⣷⡀",
    "⣿⣿⣿⣿⠃        ⣿⣿⣿⣿⡇    ⢸⣿⣿⣿⣿⡇        ⣿⣿⣿⣿⡇    ⣾⣿⣿⣿⡿         ⣿⣿⣿⣿⡇",
    "⣿⣿⣿⣿         ⣿⣿⣿⣿⡇    ⢸⣿⣿⣿⣿⡇        ⣿⣿⣿⣿⡇   ⢰⣿⣿⣿⣿⠁         ⠈⠉⠉⠉",
    "⣿⣿⣿⣿         ⣿⣿⣿⣿⡇    ⢸⣿⣿⣿⣿⡇        ⣿⣿⣿⣿⡇   ⢸⣿⣿⣿⣿",
    "⣿⣿⣿⣿         ⣿⣿⣿⣿⡇    ⢸⣿⣿⣿⣿⡇        ⣿⣿⣿⣿⡇   ⢸⣿⣿⣿⣿",
    "⣿⣿⣿⣿         ⣿⣿⣿⣿⡇    ⢸⣿⣿⣿⣿⡇        ⣿⣿⣿⣿⡇   ⠈⣿⣿⣿⣿⣆         ⣠⣤⣤⣤⡀",
    "⣿⣿⣿⣿         ⣿⣿⣿⣿⡇    ⠘⣿⣿⣿⣿⣇      ⢀⣼⣿⣿⣿⣿⡇    ⠘⣿⣿⣿⣿⣆       ⣠⣿⣿⣿⣿⠇",
    "⣿⣿⣿⣿         ⣿⣿⣿⣿⡇     ⢹⣿⣿⣿⣿⣷⣦⣤⣤⣶⣶⣿⣿⣿⣿⣿⣿⡇     ⠙⢿⣿⣿⣿⣷⣶⣤⣤⣤⣶⣾⣿⣿⣿⡿⠋",
    "⣿⣿⣿⣿         ⣿⣿⣿⣿⡇      ⠙⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠿⣿⣿⣿⣿⡇       ⠙⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠿⠋",
    "⠉⠉⠉⠉         ⠈⠉⠉⠉⠁        ⠈⠉⠉⠉⠉⠉⠉⠉  ⠈⠉⠉⠉          ⠈⠉⠉⠉⠉⠉⠉⠉⠁",
  ],
  [
    "⣿⣿⣧⣴⣾⣿⣿⣿⣷⣦    ⣾⣿⣿     ⢸⣿⣿⡆   ⢀⣴⣶⣿⣿⣿⣿⣷⣦⡀",
    "⣿⣿⣿⠟⠉⠉⠉⠛⣿⣿⣧   ⣿⣿⣿     ⢸⣿⣿⡇  ⢠⣾⣿⡿⠋⠉⠉⠙⢻⣿⣿⡆",
    "⣿⣿⡇     ⣿⣿⣿   ⣿⣿⣿     ⢸⣿⣿⡇  ⣿⣿⡿      ⠻⠿⠛",
    "⣿⣿⡇     ⣿⣿⣿   ⣿⣿⣿     ⢸⣿⣿⡇  ⣿⣿⡇",
    "⣿⣿⡇     ⣿⣿⣿   ⣿⣿⣿     ⢸⣿⣿⡇  ⣿⣿⣧      ⣀⣀⣀",
    "⣿⣿⡇     ⣿⣿⣿   ⢹⣿⣿⣆⣀⣀⣀⣠⣿⣿⣿⡇  ⠘⣿⣿⣧⣀⣀⢀⣀⣴⣿⣿⠏",
    "⣿⣿⡇     ⣿⣿⣿    ⠻⢿⣿⣿⣿⣿⡿⢿⣿⣿⠇   ⠈⠻⢿⣿⣿⣿⣿⡿⠟⠁",
  ],
  [
    "⣿⣷⡾⠿⢿⣷⡀ ⢸⣿   ⢸⣿  ⢠⣶⡿⠿⢿⣶⡄",
    "⣿⡏   ⣿⡇ ⢸⣿   ⢸⣿ ⢀⣿⡏   ⠛⠛",
    "⣿⡇   ⣿⡇ ⢸⣿   ⢸⣿ ⠸⣿⡇   ⢀⡀",
    "⣿⡇   ⣿⡇ ⠘⣿⣧⣤⣤⣾⣿  ⠻⣿⣤⣤⣴⣿⠏",
    "⠉⠁   ⠉⠁  ⠈⠉⠉⠉⠈⠉   ⠈⠉⠉⠉",
  ],
  ["⣿⠋⢻⡄⣿ ⢸⡇⢰⡟⠙⠷", "⣿ ⢸⡇⢻⣄⣼⡇⠸⣧⣠⡶"],
];

function width() {
  return Math.max(8, Math.min(120, (process.stdout.columns ?? 124) - 4));
}

function colors() {
  return styleText("red", ".") !== ".";
}

function frame(text) {
  return styleText("gray", text);
}

function wrap(text, columns) {
  return wrapAnsi(String(text).replace(/\t/g, "    "), Math.max(1, columns), {
    hard: true,
    trim: false,
  }).split("\n");
}

function pad(text, columns) {
  return text + " ".repeat(Math.max(0, columns - stringWidth(text)));
}

function block(title, lines, write = console.log) {
  const columns = width() - 4;
  const rule = "─".repeat(columns + 2);
  const row = (line) =>
    write(`  ${frame("│")} ${pad(line, columns)} ${frame("│")}`);
  write(`\n  ${frame(`┌${rule}┐`)}`);
  if (title !== "") {
    for (const line of wrap(stripVTControlCharacters(String(title)), columns)) {
      row(styleText(["bold", "white"], line));
    }
    write(`  ${frame(`├${rule}┤`)}`);
  }
  for (const line of lines) {
    for (const wrapped of wrap(line, columns)) {
      row(styleText("white", wrapped));
    }
  }
  write(`  ${frame(`└${rule}┘`)}`);
}

export function header({ version }) {
  const available = width() - 4;
  const horizontal = available >= 32;
  const limit = Math.min(12, available);
  const wordmark = wordmarks.find((art) =>
    art.every((line) => stringWidth(line) <= limit)
  ) ?? ["nuc"];
  const logo = wordmark.map((line) => styleText("green", line));
  const logoWidth = Math.max(...logo.map((line) => stringWidth(line)));
  const textWidth = horizontal ? available - logoWidth - 3 : available;
  const text = [
    ...wrap("Nucleoid - Logic Language for World Models 🌱🐋🌍", textWidth).map(
      (line) => styleText(["bold", "white"], line)
    ),
    ...wrap("Inspired by Nature", textWidth).map((line) =>
      styleText("gray", line)
    ),
    "",
    ...wrap("Analyze / Construct / Visualize", textWidth),
    ...wrap(`Version ${version}`, textWidth).map((line) =>
      styleText("gray", line)
    ),
  ];
  const height = Math.max(logo.length, text.length);
  const lines = horizontal
    ? Array.from(
        { length: height },
        (_, index) =>
          pad(logo[index] ?? "", logoWidth) + "   " + (text[index] ?? "")
      )
    : [...logo, "", ...text];
  const contentWidth = Math.max(...lines.map((line) => stringWidth(line)));
  const inset = " ".repeat(
    Math.max(0, Math.floor((available - contentWidth) * 0.5))
  );
  block("", ["", "", ...lines.map((line) => inset + line), "", ""]);
  console.log("");
}

export function section(index, title, detail) {
  const [label, ...continuation] = wrap(
    `${String(index).padStart(2, "0")}  ${title.toUpperCase()}`,
    width() - 4
  );
  const rule = "─".repeat(Math.max(0, width() - stringWidth(label) - 3));
  console.log(
    `\n  ${frame("◆")} ${styleText(["bold", "white"], label)} ${frame(rule)}`
  );
  for (const line of continuation) {
    console.log(`    ${styleText(["bold", "white"], line)}`);
  }
  for (const line of wrap(detail, width() - 2)) {
    console.log(`    ${styleText("gray", line)}`);
  }
  console.log("");
}

export function step(message) {
  const [first, ...continuation] = wrap(
    stripVTControlCharacters(String(message)),
    width() - 2
  );
  console.log(`  ${frame("›")} ${styleText("white", first)}`);
  for (const line of continuation) {
    console.log(`    ${styleText("white", line)}`);
  }
}

export function code(title, source) {
  const text = Array.isArray(source) ? source.join("\n\n") : String(source);
  block(
    title,
    stripVTControlCharacters(text)
      .split("\n")
      .map((line) => styleText("whiteBright", line))
  );
}

export function value(title, data) {
  block(
    title,
    inspect(data, {
      colors: false,
      depth: null,
      compact: false,
      breakLength: Math.max(8, width() - 6),
      maxArrayLength: null,
      maxStringLength: null,
    })
      .split("\n")
      .map((line) => styleText("whiteBright", line))
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

function matrixLines(matrix, start, end, colored) {
  if (matrix.length === 0 || start === end) {
    return [styleText("gray", "(empty)")];
  }
  const rowWidth = String(matrix.length - 1).length;
  const axis =
    " ".repeat(rowWidth + 2) +
    Array.from({ length: end - start }, (_, index) =>
      `${start + index}`.padStart(2).concat(" ")
    ).join("");
  return [
    styleText("gray", axis),
    ...matrix.map(
      (row, index) =>
        styleText("gray", `${String(index).padStart(rowWidth)}  `) +
        row
          .slice(start, end)
          .map((value) => cell(value, colored))
          .join("")
    ),
  ];
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
    block(`${title} · ${dimensions}`, [
      ...(columns > chunk
        ? [styleText("gray", `Columns ${start}–${end - 1}`), ""]
        : []),
      ...matrixLines(matrix, start, end, colored),
    ]);
  }
}

function sideBySide(panels) {
  const widths = panels.map((lines) =>
    Math.max(...lines.map((line) => stringWidth(line)))
  );
  const height = Math.max(...panels.map((lines) => lines.length));
  return Array.from({ length: height }, (_, row) =>
    panels
      .map((lines, index) => pad(lines[row] ?? "", widths[index]))
      .join("  ")
  );
}

function previewMatrix(title, matrix, colored) {
  const columns = Math.max(0, ...matrix.map((row) => row.length));
  return [
    styleText("gray", `${title} · ${columns} × ${matrix.length}`),
    ...matrixLines(matrix, 0, columns, colored),
  ];
}

export function puzzle({ train, testInput }) {
  const available = width() - 4;
  const colored = colors();
  let pairs = [];
  let used = 0;
  const renderRow = () => {
    if (pairs.length === 0) {
      return;
    }
    block("Training", sideBySide(pairs));
    pairs = [];
    used = 0;
  };

  if (train.length === 0) {
    block("Training", [styleText("gray", "(empty)")]);
  }
  for (const [index, { input, output }] of train.entries()) {
    const lines = [
      styleText(["bold", "white"], `Train ${index + 1}`),
      ...sideBySide([
        previewMatrix("Input", input, colored),
        previewMatrix("Output", output, colored),
      ]),
    ];
    const pairWidth = Math.max(...lines.map((line) => stringWidth(line)));
    if (pairWidth > available) {
      renderRow();
      grid(`Train ${index + 1} · Input`, input);
      grid(`Train ${index + 1} · Output`, output);
      continue;
    }
    if (pairs.length > 0 && used + 2 + pairWidth > available) {
      renderRow();
    }
    used += (pairs.length > 0 ? 2 : 0) + pairWidth;
    pairs.push(lines);
  }
  renderRow();
  grid("Test input", testInput);
}

export function usage({ prompt_tokens, completion_tokens }) {
  const message = `↳ Model usage  ${prompt_tokens} input / ${completion_tokens} output tokens`;
  for (const line of wrap(message, width() - 2)) {
    console.log(`    ${styleText("gray", line)}`);
  }
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
      : styleText(["bold", "red"], "◇ MISMATCH");
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
