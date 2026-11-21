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

interface Milestone {
  key: string;
  loop?: boolean;
}

let stageNumber = 0;
const milestones = new Map<string, { number: number; iteration: number }>();

function numbered(title: string, milestone?: Milestone): string {
  if (stageNumber === 0) {
    return title;
  }
  const key: string = milestone?.key ?? title;
  const counter = milestones.get(key) ?? {
    number: milestones.size + 1,
    iteration: 0,
  };
  counter.iteration += 1;
  milestones.set(key, counter);
  const iteration: string = milestone?.loop ? `.${counter.iteration}` : "";
  return `${stageNumber}.${counter.number}${iteration} ${title}`;
}

function width(): number {
  return Math.max(8, Math.min(120, (process.stdout.columns ?? 124) - 4));
}

function colors(): boolean {
  return styleText("red", ".") !== ".";
}

function frame(text: string): string {
  return styleText("gray", text);
}

function expand(text: string): string {
  return String(text).replace(/\t/g, "    ");
}

function wrap(text: string, columns: number) {
  return wrapAnsi(expand(text), Math.max(1, columns), {
    hard: true,
    trim: false,
  }).split("\n");
}

function pad(text: string, columns: number): string {
  return text + " ".repeat(Math.max(0, columns - stringWidth(text)));
}

function block(title: string, lines, write = console.log) {
  const columns: number = width() - 4;
  const rule: string = "─".repeat(columns + 2);
  const output = write === console.error ? process.stderr : process.stdout;
  const rightBorder: string = output.isTTY ? `\x1b[${columns + 6}G` : " ";
  const row = (line: string) => {
    const content: string = output.isTTY ? line : pad(line, columns);
    write(`  ${frame("│")} ${content}${rightBorder}${frame("│")}`);
  };
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

export function header({
  version,
  puzzle_id,
}: {
  version: string;
  puzzle_id: string;
}) {
  stageNumber = 0;
  milestones.clear();
  const available: number = width() - 4;
  const horizontal: boolean = available >= 32;
  const limit: number = Math.min(12, available);
  const wordmark = wordmarks.find((art) =>
    art.every((line: string) => stringWidth(line) <= limit)
  ) ?? ["nuc"];
  const logo = wordmark.map((line: string) => styleText("greenBright", line));
  const logoWidth: number = Math.max(
    ...logo.map((line: string) => stringWidth(line))
  );
  const textWidth: number = horizontal ? available - logoWidth - 3 : available;
  const text = [
    ...wrap("Nucleoid - Logic Language for World Models 🌱🐋🌍", textWidth).map(
      (line: string) => styleText(["bold", "white"], line)
    ),
    ...wrap("Inspired by Nature", textWidth).map((line: string) =>
      styleText("gray", line)
    ),
    "",
    ...wrap("Analyze / Construct / Visualize", textWidth),
    ...wrap(`Version ${version}`, textWidth).map((line: string) =>
      styleText("gray", line)
    ),
    ...wrap(`Run time (UTC) ${new Date().toISOString()}`, textWidth).map(
      (line: string) => styleText("gray", line)
    ),
    ...wrap(`Puzzle ${puzzle_id}`, textWidth).map((line: string) =>
      styleText("gray", line)
    ),
  ];
  const height: number = Math.max(logo.length, text.length);
  const lines = horizontal
    ? Array.from(
        { length: height },
        (_, index: number) =>
          pad(logo[index] ?? "", logoWidth) + "   " + (text[index] ?? "")
      )
    : [...logo, "", ...text];
  const contentWidth: number = Math.max(
    ...lines.map((line: string) => stringWidth(line))
  );
  const inset: string = " ".repeat(
    Math.max(0, Math.floor((available - contentWidth) * 0.5))
  );
  block("", ["", "", ...lines.map((line: string) => inset + line), "", ""]);
  console.log("");
}

export function section(index: number, title: string, detail: string) {
  stageNumber = index;
  milestones.clear();
  const [label, ...continuation] = wrap(
    `${String(index).padStart(2, "0")}  ${title.toUpperCase()}`,
    width() - 4
  );
  const rule: string = "─".repeat(
    Math.max(0, width() - stringWidth(label) - 3)
  );
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

function bullet(
  marker: string,
  style: Parameters<typeof styleText>[0],
  message: string,
  milestone?: Milestone
) {
  const [first, ...continuation] = wrap(
    numbered(stripVTControlCharacters(String(message)), milestone),
    width() - 2
  );
  console.log(`  ${marker} ${styleText(style, first)}`);
  for (const line of continuation) {
    console.log(`    ${styleText(style, line)}`);
  }
}

export function step(message: string, milestone?: Milestone) {
  bullet(frame("›"), "white", message, milestone);
}

export function rejection(message: string, milestone?: Milestone) {
  bullet(styleText(["bold", "red"], "✗"), "red", message, milestone);
}

function codeLines(source): string[] {
  const text: string = Array.isArray(source)
    ? source.join("\n\n")
    : String(source);
  return stripVTControlCharacters(text)
    .split("\n")
    .map((line: string) => styleText("whiteBright", line));
}

function valueLines(data): string[] {
  return inspect(data, {
    colors: false,
    depth: null,
    compact: false,
    breakLength: Math.max(8, width() - 6),
    maxArrayLength: null,
    maxStringLength: null,
  })
    .split("\n")
    .map((line: string) => styleText("whiteBright", line));
}

export function code(title: string, source, milestone?: Milestone) {
  block(numbered(title, milestone), codeLines(source));
}

export function value(title: string, data, milestone?: Milestone) {
  block(numbered(title, milestone), valueLines(data));
}

function span(widths): number {
  return widths.reduce((sum: number, cell: number) => sum + cell + 1, -1);
}

function minimumWidth(text: string): number {
  return Array.from(stripVTControlCharacters(text)).reduce(
    (minimum: number, character: string) =>
      Math.max(minimum, stringWidth(character)),
    1
  );
}

function tableRule(widths, left: string, joint: string, right: string): string {
  return `  ${frame(
    left + widths.map((cell: number) => "─".repeat(cell)).join(joint) + right
  )}`;
}

function tableRow(widths, cells) {
  const lines = cells.map((text: string, index: number) =>
    wrap(text, widths[index])
  );
  const height: number = Math.max(...lines.map((cell) => cell.length));
  return Array.from(
    { length: height },
    (_, line: number) =>
      `  ${frame("│")}${lines
        .map((cell, index: number) => pad(cell[line] ?? "", widths[index]))
        .join(frame("│"))}${frame("│")}`
  );
}

function tableValue(column: string, value: unknown): string {
  const formatted: string =
    (column === "orientation" ? orientation(value) : undefined) ??
    inspect(value, {
      colors: false,
      depth: null,
      compact: true,
      breakLength: Infinity,
      maxArrayLength: null,
      maxStringLength: null,
    });
  return column === "time" ? `⏱️ ${formatted}` : formatted;
}

export function table(title: string, columns, rows, milestone?: Milestone) {
  const heading: string = numbered(title, milestone);
  if (rows.length === 0) {
    block(heading, [styleText("gray", "(empty)")]);
    return;
  }

  const header = columns.map((column: string) =>
    styleText(["bold", "white"], column)
  );
  const body = rows.map((row) =>
    columns.map((column: string) =>
      Object.hasOwn(row, column)
        ? styleText("whiteBright", tableValue(column, row[column]))
        : styleText("gray", "—")
    )
  );
  const content = [header, ...body];
  const natural = columns.map((_, column: number) =>
    Math.max(...content.map((cells) => stringWidth(cells[column])))
  );
  const widths = columns.map((_, column: number) =>
    Math.max(...content.map((cells) => minimumWidth(cells[column])))
  );
  const inner: number = Math.max(
    span(widths),
    Math.min(width() - 2, Math.max(span(natural), stringWidth(heading) + 2))
  );
  let remaining: number = inner - span(widths);
  while (remaining > 0) {
    const growable = widths.flatMap((cell: number, column: number) =>
      cell < natural[column] ? [column] : []
    );
    if (growable.length === 0) {
      break;
    }
    for (const column of growable.slice(0, remaining)) {
      widths[column] += 1;
      remaining -= 1;
    }
  }
  widths[widths.length - 1] += remaining;

  const titleWidth: number = span(widths) - 2;
  console.log(`\n  ${frame(`┌${"─".repeat(titleWidth + 2)}┐`)}`);
  for (const line of wrap(stripVTControlCharacters(heading), titleWidth)) {
    console.log(
      `  ${frame("│")} ${pad(
        styleText(["bold", "white"], line),
        titleWidth
      )} ${frame("│")}`
    );
  }
  console.log(tableRule(widths, "├", "┬", "┤"));
  for (const line of tableRow(widths, header)) {
    console.log(line);
  }
  console.log(tableRule(widths, "├", "┼", "┤"));
  for (const cells of body) {
    for (const line of tableRow(widths, cells)) {
      console.log(line);
    }
  }
  console.log(tableRule(widths, "└", "┴", "┘"));
}

function cell(value: number, colored: boolean): string {
  const text: string = `${String(value).padStart(2)} `;
  if (!colored || !palette[value]) {
    return text;
  }
  const [red, green, blue] = palette[value];
  const foreground: number =
    red * 0.299 + green * 0.587 + blue * 0.114 > 160 ? 30 : 97;
  return `\x1b[48;2;${red};${green};${blue}m\x1b[${foreground}m${text}\x1b[0m`;
}

function matrixLines(matrix, start: number, end: number, colored: boolean) {
  if (matrix.length === 0 || start === end) {
    return [styleText("gray", "(empty)")];
  }
  const rowWidth: number = String(matrix.length - 1).length;
  const axis: string =
    " ".repeat(rowWidth + 2) +
    Array.from({ length: end - start }, (_, index: number) =>
      `${start + index}`.padStart(2).concat(" ")
    ).join("");
  return [
    styleText("gray", axis),
    ...matrix.map(
      (row, index: number) =>
        styleText("gray", `${String(index).padStart(rowWidth)}  `) +
        row
          .slice(start, end)
          .map((value: number) => cell(value, colored))
          .join("")
    ),
  ];
}

const compass: string[] = ["⬆️", "↗️", "➡️", "↘️", "⬇️", "↙️", "⬅️", "↖️"];

export function orientation(degrees: unknown): string | undefined {
  if (typeof degrees !== "number" || !Number.isFinite(degrees)) {
    return undefined;
  }
  const turn: number = ((degrees % 360) + 360) % 360;
  return `${compass[turn / 45] ?? "🧭"} ${degrees}°`;
}

export function grid(title: string, matrix, milestone?: Milestone) {
  const heading: string = numbered(title, milestone);
  const rows: number = matrix.length;
  const columns: number = Math.max(0, ...matrix.map((row) => row.length));
  if (rows === 0 || columns === 0) {
    block(heading, [styleText("gray", "(empty)")]);
    return;
  }

  const rowWidth: number = String(rows - 1).length;
  const chunk: number = Math.max(1, Math.floor((width() - rowWidth - 6) / 3));
  const colored: boolean = colors();
  for (let start: number = 0; start < columns; start += chunk) {
    const end: number = Math.min(columns, start + chunk);
    block(heading, [
      ...(columns > chunk
        ? [styleText("gray", `Columns ${start}–${end - 1}`), ""]
        : []),
      ...matrixLines(matrix, start, end, colored),
    ]);
  }
}

export type Panel =
  | { kind: "grid"; title: string; matrix: number[][] }
  | { kind: "text"; title: string; lines: string[] };

const textMinimum: number = 40;
const titleMinimum: number = 32;
const panelGap: number = 3;

export function gridPanel(title: string, matrix, milestone?: Milestone): Panel {
  return { kind: "grid", title: numbered(title, milestone), matrix };
}

export function codePanel(title: string, source, milestone?: Milestone): Panel {
  return {
    kind: "text",
    title: numbered(title, milestone),
    lines: codeLines(source),
  };
}

export function valuePanel(title: string, data, milestone?: Milestone): Panel {
  return {
    kind: "text",
    title: numbered(title, milestone),
    lines: valueLines(data),
  };
}

function gridWidth(matrix): number {
  const rows: number = matrix.length;
  const columns: number = Math.max(0, ...matrix.map((row) => row.length));
  return rows === 0 || columns === 0
    ? stringWidth("(empty)")
    : String(rows - 1).length + 2 + columns * 3;
}

function gridLines(matrix, available: number, colored: boolean): string[] {
  const rows: number = matrix.length;
  const columns: number = Math.max(0, ...matrix.map((row) => row.length));
  if (rows === 0 || columns === 0) {
    return [styleText("gray", "(empty)")];
  }
  const chunk: number = Math.max(
    1,
    Math.floor((available - String(rows - 1).length - 2) / 3)
  );
  if (columns <= chunk) {
    return matrixLines(matrix, 0, columns, colored);
  }
  return Array.from({ length: Math.ceil(columns / chunk) }, (_, index) => {
    const start: number = index * chunk;
    const end: number = Math.min(columns, start + chunk);
    return [
      ...(index > 0 ? [""] : []),
      styleText("gray", `Columns ${start}–${end - 1}`),
      "",
      ...matrixLines(matrix, start, end, colored),
    ];
  }).flat();
}

function panelContent(panel: Panel): number {
  return panel.kind === "grid"
    ? gridWidth(panel.matrix)
    : Math.max(
        0,
        ...panel.lines.map((line: string) => stringWidth(expand(line)))
      );
}

function panelNatural(panel: Panel): number {
  return Math.max(panelContent(panel), stringWidth(panel.title));
}

function panelMinimum(panel: Panel, available: number): number {
  const content: number = panelContent(panel);
  return Math.min(
    available,
    Math.max(
      panel.kind === "grid" ? content : Math.min(content, textMinimum),
      Math.min(stringWidth(panel.title), titleMinimum)
    )
  );
}

function panelSpan(widths: number[]): number {
  return widths.reduce(
    (sum: number, cell: number, index: number) =>
      sum + cell + (index > 0 ? panelGap : 0),
    0
  );
}

function panelRows(panels: Panel[], available: number): Panel[][] {
  const rows: Panel[][] = [];
  let used: number = 0;
  for (const panel of panels) {
    const minimum: number = panelMinimum(panel, available);
    if (rows.length > 0 && used + panelGap + minimum <= available) {
      rows[rows.length - 1].push(panel);
      used += panelGap + minimum;
    } else {
      rows.push([panel]);
      used = minimum;
    }
  }
  return rows;
}

function panelWidths(row: Panel[], available: number, target: number) {
  const naturals: number[] = row.map(panelNatural);
  const widths: number[] = row.map((panel: Panel) =>
    panelMinimum(panel, available)
  );
  let remaining: number = target - panelSpan(widths);
  while (remaining > 0) {
    const growable: number[] = widths.flatMap((cell: number, column: number) =>
      cell < naturals[column] ? [column] : []
    );
    if (growable.length === 0) {
      break;
    }
    for (const column of growable.slice(0, remaining)) {
      widths[column] += 1;
      remaining -= 1;
    }
  }
  return widths;
}

function panelRules(widths: number[]): number[] {
  return widths.map(
    (_, index: number) => panelSpan(widths.slice(0, index + 1)) + 2
  );
}

function panelRule(
  inner: number,
  upper: number[],
  lower: number[],
  left: string,
  right: string
): string {
  const line: string = Array.from({ length: inner + 2 }, (_, offset) => {
    const above: boolean = upper.includes(offset);
    const below: boolean = lower.includes(offset);
    return above ? (below ? "┼" : "┴") : below ? "┬" : "─";
  }).join("");
  return `  ${frame(left + line + right)}`;
}

function panelLines(widths: number[], cells: string[][], tty: boolean) {
  const rules: number[] = panelRules(widths);
  const height: number = Math.max(1, ...cells.map((lines) => lines.length));
  return Array.from({ length: height }, (_, row: number) =>
    cells
      .map((lines: string[], index: number) => {
        const text: string = lines[row] ?? "";
        return tty
          ? ` ${text}\x1b[${rules[index] + 4}G${frame("│")}`
          : ` ${pad(text, widths[index])} ${frame("│")}`;
      })
      .join("")
  ).map((line: string) => `  ${frame("│")}${line}`);
}

function panelWrap(text: string, columns: number): string[] {
  return wrap(text, columns).map((row: string, index: number) => {
    const space: number = row.indexOf(" ");
    return index > 0 &&
      space >= 0 &&
      stripVTControlCharacters(row.slice(0, space)) === ""
      ? row.slice(0, space) + row.slice(space + 1)
      : row;
  });
}

function panelHeading(title: string, columns: number): string[] {
  return panelWrap(stripVTControlCharacters(title), columns).map(
    (line: string) => styleText(["bold", "white"], line)
  );
}

function panelBody(panel: Panel, columns: number, colored: boolean) {
  const lines: string[] =
    panel.kind === "grid"
      ? gridLines(panel.matrix, columns, colored)
      : panel.lines;
  return lines.flatMap((line: string) => panelWrap(line, columns));
}

export function panels(title: string, groups: Panel[][]) {
  const available: number = width() - 4;
  const rows: Panel[][] = groups.flatMap((group: Panel[]) =>
    panelRows(group, available)
  );
  if (rows.length === 0) {
    return;
  }

  const inner: number = Math.max(
    Math.min(stringWidth(title), available),
    ...rows.map((row: Panel[]) =>
      panelSpan(panelWidths(row, available, available))
    )
  );
  const widths: number[][] = rows.map((row: Panel[]) => {
    const cells: number[] = panelWidths(row, available, inner);
    cells[cells.length - 1] += inner - panelSpan(cells);
    return cells;
  });
  const rules: number[][] = widths.map((cells: number[]) =>
    panelRules(cells).slice(0, -1)
  );
  const tty: boolean = process.stdout.isTTY === true;
  const colored: boolean = colors();

  const lines: string[] = [
    panelRule(inner, [], title === "" ? rules[0] : [], "┌", "┐"),
  ];
  if (title !== "") {
    lines.push(
      ...panelLines([inner], [panelHeading(title, inner)], tty),
      panelRule(inner, [], rules[0], "├", "┤")
    );
  }
  rows.forEach((row: Panel[], index: number) => {
    const cells: number[] = widths[index];
    lines.push(
      ...panelLines(
        cells,
        row.map((panel: Panel, column: number) =>
          panelHeading(panel.title, cells[column])
        ),
        tty
      ),
      panelRule(inner, rules[index], rules[index], "├", "┤"),
      ...panelLines(
        cells,
        row.map((panel: Panel, column: number) =>
          panelBody(panel, cells[column], colored)
        ),
        tty
      ),
      index + 1 < rows.length
        ? panelRule(inner, rules[index], rules[index + 1], "├", "┤")
        : panelRule(inner, rules[index], [], "└", "┘")
    );
  });
  console.log(`\n${lines.join("\n")}`);
}

function sideBySide(panels) {
  const widths = panels.map((lines) =>
    Math.max(...lines.map((line: string) => stringWidth(line)))
  );
  const height: number = Math.max(...panels.map((lines) => lines.length));
  return Array.from({ length: height }, (_, row: number) =>
    panels
      .map((lines, index: number) => pad(lines[row] ?? "", widths[index]))
      .join("  ")
  );
}

function previewMatrix(title: string, matrix, colored: boolean) {
  const columns: number = Math.max(0, ...matrix.map((row) => row.length));
  return [
    styleText("gray", title),
    ...matrixLines(matrix, 0, columns, colored),
  ];
}

export function puzzle({ train, testInput }) {
  const available: number = width() - 4;
  const colored: boolean = colors();
  let pairs = [];
  let used: number = 0;
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
    const pairWidth: number = Math.max(
      ...lines.map((line: string) => stringWidth(line))
    );
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

export function reasoning(text: unknown) {
  const content: string =
    typeof text === "string" ? stripVTControlCharacters(text).trim() : "";
  const heading: string =
    content === "" ? "↳ Model reasoning  none returned" : "↳ Model reasoning";
  for (const line of wrap(heading, width() - 2)) {
    console.log(`    ${styleText("gray", line)}`);
  }
  if (content === "") {
    return;
  }
  for (const step of content.split(/\r?\n/)) {
    for (const line of wrap(step, width() - 4)) {
      console.log(`      ${styleText("gray", line)}`);
    }
  }
}

export function usage({ prompt_tokens, completion_tokens, cached_tokens }) {
  const cached: string =
    typeof cached_tokens === "number" ? ` (${cached_tokens} cached)` : "";
  const message: string = `↳ Model usage  ${prompt_tokens} input${cached} / ${completion_tokens} output tokens`;
  for (const line of wrap(message, width() - 2)) {
    console.log(`    ${styleText("gray", line)}`);
  }
}

export function alert(message: string) {
  for (const line of wrap(`⚠ ${message}`, width() - 2)) {
    console.warn(`    ${styleText(["bold", "yellow"], line)}`);
  }
}

function differenceLines(difference: number, columns: number): string[] {
  const digits: string[] =
    String(difference)
      .padStart(2, "0")
      .replace(/\d/g, (digit: string) => `${digit}\ufe0f\u20e3`)
      .match(/\d\ufe0f\u20e3|./gu) ?? [];
  const prefix: string = "Difference  ";
  const digitWidth: number = process.platform === "win32" ? 3 : 2;
  if (prefix.length + digits.length * digitWidth <= columns) {
    return [prefix + digits.join("")];
  }
  const perLine: number = Math.max(1, Math.floor(columns / digitWidth));
  return [
    prefix.trimEnd(),
    ...Array.from(
      { length: Math.ceil(digits.length / perLine) },
      (_, index: number) =>
        digits.slice(index * perLine, (index + 1) * perLine).join("")
    ),
  ];
}

export function summary(difference: number, duration: number) {
  const seconds: number = duration / 1000;
  const elapsed: string =
    seconds < 60
      ? `${seconds.toFixed(1)}s`
      : `${Math.floor(seconds / 60)}m ${(seconds % 60).toFixed(1)}s`;
  const status: string =
    difference === 0
      ? styleText(["bold", "green"], "✓ EXACT MATCH")
      : styleText(["bold", "red"], "◇ MISMATCH");
  block(numbered("Run complete"), [
    status,
    "",
    ...differenceLines(difference, width() - 4),
    `Elapsed     ⏱️ ${elapsed}`,
  ]);
  console.log("");
}

export function failure(error) {
  const text: string = inspect(error, {
    colors: false,
    depth: null,
    maxArrayLength: null,
    maxStringLength: null,
  });
  block(
    numbered("Run failed"),
    stripVTControlCharacters(text)
      .split("\n")
      .map((line: string) => styleText("red", line)),
    console.error
  );
}
