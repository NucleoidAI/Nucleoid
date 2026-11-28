import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { document } from "./dataset";
import { palette } from "../lib/terminal";
import * as pdf from "../lib/pdf";

const output_path: string = join(__dirname, "..", "..", "dataset.pdf");
const document_title: string = "Nucleoid ARC instruction datasets";
const introduction = [
  "Every JSON instruction dataset in arc/src/dataset, in the filename order in which document() reads them for the solver's few-shot examples. Generated from the dataset files by bun run dataset:pdf in arc/; regenerate it whenever a dataset changes instead of editing it.",
  "Each dataset shows its class_declaration, its declarations and its output_canvas, then one box per pair in dataset order: the pair's input_matrix and output_matrix, followed by one row per instance with its input_instance_matrix, its output_instance_matrix and its implementation. Sizes read rows×columns, and cells show colors 0 to 9 in the solver's terminal palette.",
];

const page_width: number = 612;
const page_height: number = 792;
const margin: number = 40;
const content_top: number = 52;
const content_bottom: number = page_height - 48;
const content_width: number = page_width - 2 * margin;
const content_right: number = margin + content_width;
const header_baseline: number = 30;
const header_rule: number = 36;
const footer_baseline: number = page_height - 26;

const padding: number = 8;
const title_height: number = 16;
const title_baseline: number = 11;
const box_gap: number = 10;
const panel_gap: number = 8;
const label_height: number = 10;
const code_leading: number = 9;
const code_padding: number = 5;
const code_chunk: number = 60;
const code_hanging: number = 8;
const wrap_columns_minimum: number = 16;
const gutter_width: number = 16;
const value_height: number = 20;
const implementation_leading: number = 8.6;
const implementation_hanging: number = 4;
const instance_padding: number = 6;
const beside_share: number = 0.45;
const beside_columns: number = 56;
const beside_height_maximum: number = 400;
const grid_arrow: number = 22;
const grid_cell_minimum: number = 2;
const grid_cell_maximum: number = 14;
const digit_cell_minimum: number = 9;
const grid_height_maximum: number = 220;
const thumbnail_arrow: number = 14;
const thumbnail_extent: number = 44;
const thumbnail_cell_minimum: number = 1.5;
const thumbnail_cell_maximum: number = 8;
const swatch_size: number = 14;
const swatch_gap: number = 6;
const contents_row: number = 13;
const contents_columns = {
  dataset: margin + 6,
  canvas: margin + 210,
  pairs: margin + 330,
  instances: margin + 410,
  page: content_right - 6,
};

const ink = [33, 37, 41];
const muted = [108, 117, 125];
const rule = [206, 212, 218];
const title_fill = [241, 243, 245];
const code_fill = [250, 251, 252];
const stripe_fill = [248, 249, 250];
const grid_line = [85, 85, 85];
const black = [0, 0, 0];
const white = [255, 255, 255];

const title_style = { font: "sans_bold", size: 20, color: ink };
const heading_style = { font: "sans_bold", size: 16, color: ink };
const body_style = { font: "sans", size: 8.5, color: ink };
const total_style = { font: "sans_bold", size: 8.5, color: ink };
const summary_style = { font: "sans", size: 8.5, color: muted };
const box_style = { font: "sans_bold", size: 7.5, color: ink };
const note_style = { font: "sans", size: 7, color: muted };
const label_style = { font: "sans", size: 6.5, color: muted };
const code_style = { font: "mono", size: 7, color: ink };
const gutter_style = { font: "mono", size: 7, color: muted };
const value_style = { font: "mono_bold", size: 8, color: ink };
const implementation_style = { font: "mono", size: 6.8, color: ink };
const name_style = { font: "mono_bold", size: 6.8, color: ink };
const column_style = { font: "sans_bold", size: 7.5, color: ink };
const file_style = { font: "mono", size: 7.5, color: ink };
const count_style = { font: "sans", size: 7.5, color: ink };

function plural(count: number, noun: string): string {
  return `${count} ${noun}${count === 1 ? "" : "s"}`;
}

function clamp(value: number, low: number, high: number): number {
  return Math.min(high, Math.max(low, value));
}

function sum(values): number {
  return values.reduce((total: number, value: number) => total + value, 0);
}

function puzzle_id(file: string): string {
  return /^dataset\.([0-9a-f]{8})\.json$/.exec(file.toLowerCase())?.[1];
}

function instance_count(dataset): number {
  return sum(dataset.dataset.map((pair) => pair.instances.length));
}

function parse(file: string) {
  const source: string = readFileSync(join(__dirname, file), "utf8");
  try {
    return JSON.parse(source);
  } catch (error) {
    throw new Error(`${file}: ${error.message}`);
  }
}

function load() {
  const files: string[] = readdirSync(__dirname, { withFileTypes: true })
    .filter((file) => {
      const name: string = file.name.toLowerCase();
      return (
        file.isFile() && name.endsWith(".json") && !name.startsWith(".env")
      );
    })
    .map((file) => file.name)
    .sort();
  const datasets = files.map(parse);
  if (JSON.stringify(datasets) !== JSON.stringify(JSON.parse(document()))) {
    throw new Error(
      "The dataset files differ from the instruction datasets document() reads"
    );
  }
  return files.map((file: string, index: number) => ({
    file,
    dataset: datasets[index],
  }));
}

function wrap_text(text: string, style, width: number) {
  const lines = [];
  let line: string = "";
  for (const word of text.split(" ")) {
    const candidate: string = line === "" ? word : `${line} ${word}`;
    if (line !== "" && pdf.text_width(candidate, style) > width) {
      lines.push(line);
      line = word;
    } else {
      line = candidate;
    }
  }
  return line === "" ? lines : [...lines, line];
}

function wrap_code(line: string, available: number, hanging: number) {
  const columns: number = Math.max(available, wrap_columns_minimum);
  const leading: number = line.length - line.trimStart().length;
  const indent: string = " ".repeat(
    Math.min(Math.floor(columns / 2), leading + hanging)
  );
  const lines = [];
  let rest: string = line;
  let prefix: string = "";
  while (prefix.length + rest.length > columns) {
    const room: number = columns - prefix.length;
    const start: number = rest.length - rest.trimStart().length;
    const space: number = rest.lastIndexOf(" ", room);
    const cut: number = space > start ? space : room;
    lines.push(prefix + rest.slice(0, cut).trimEnd());
    rest = rest.slice(cut).trimStart();
    prefix = indent;
  }
  return rest === "" && lines.length > 0 ? lines : [...lines, prefix + rest];
}

function code_columns(width: number, style): number {
  return Math.floor(width / pdf.text_width(" ", style));
}

function grid_size(matrix) {
  return {
    rows: matrix.length,
    columns: Math.max(0, ...matrix.map((row) => row.length)),
  };
}

function color_of(value: number) {
  return palette[value] ?? white;
}

function digit_style(value: number, cell: number) {
  const [red, green, blue] = color_of(value);
  return {
    font: "sans_bold",
    size: cell * 0.5,
    color: red * 0.299 + green * 0.587 + blue * 0.114 > 160 ? black : white,
  };
}

function draw_digit(page, x: number, y: number, cell: number, value: number) {
  const text: string = String(value);
  const style = digit_style(value, cell);
  pdf.text(
    page,
    x + (cell - pdf.text_width(text, style)) / 2,
    y + cell / 2 + style.size * 0.36,
    text,
    style
  );
}

function draw_grid(
  page,
  x: number,
  y: number,
  matrix,
  cell: number,
  digits: boolean
) {
  const { rows, columns } = grid_size(matrix);
  if (rows === 0 || columns === 0) {
    pdf.text(page, x, y + label_style.size, "(empty)", label_style);
    return;
  }
  const width: number = columns * cell;
  const height: number = rows * cell;
  pdf.rects(page, [[x, y, width, height]], color_of(0));
  const cells = new Map();
  matrix.forEach((row, r: number) =>
    row.forEach((value: number, c: number) => {
      if (value !== 0) {
        const areas = cells.get(value) ?? [];
        areas.push([x + c * cell, y + r * cell, cell, cell]);
        cells.set(value, areas);
      }
    })
  );
  for (const [value, areas] of cells) {
    pdf.rects(page, areas, color_of(value));
  }
  pdf.lines(
    page,
    [
      ...Array.from({ length: rows + 1 }, (_, r: number) => [
        x,
        y + r * cell,
        x + width,
        y + r * cell,
      ]),
      ...Array.from({ length: columns + 1 }, (_, c: number) => [
        x + c * cell,
        y,
        x + c * cell,
        y + height,
      ]),
    ],
    grid_line,
    cell >= 8 ? 0.4 : 0.25
  );
  if (digits && cell >= digit_cell_minimum) {
    for (const [value, areas] of cells) {
      for (const [left, top] of areas) {
        draw_digit(page, left, top, cell, value);
      }
    }
  }
}

function draw_arrow(page, x: number, y: number, width: number) {
  pdf.lines(page, [[x, y, x + width - 4, y]], muted, 0.8);
  pdf.polygon(
    page,
    [
      [x + width, y],
      [x + width - 5, y - 2.8],
      [x + width - 5, y + 2.8],
    ],
    muted
  );
}

function draw_right(
  page,
  right: number,
  baseline: number,
  text: string,
  style
) {
  pdf.text(page, right - pdf.text_width(text, style), baseline, text, style);
}

function code_rows(source: string, gutter?: string) {
  const offset: number = gutter === undefined ? 0 : gutter_width;
  const columns: number = code_columns(
    content_width - 2 * padding - offset,
    code_style
  );
  const lines = source
    .split("\n")
    .flatMap((line: string) =>
      wrap_code(line.replace(/\t/g, "    "), columns, code_hanging)
    );
  return Array.from(
    { length: Math.ceil(lines.length / code_chunk) },
    (_, chunk: number) => {
      const part = lines.slice(chunk * code_chunk, (chunk + 1) * code_chunk);
      const height: number = part.length * code_leading + 2 * code_padding;
      return {
        height,
        draw: (page, x: number, y: number, width: number) => {
          const baseline: number = y + code_padding + code_style.size;
          pdf.rects(page, [[x, y, width, height]], code_fill);
          if (gutter !== undefined && chunk === 0) {
            pdf.text(page, x + padding, baseline, gutter, gutter_style);
          }
          part.forEach((line: string, index: number) =>
            pdf.text(
              page,
              x + padding + offset,
              baseline + index * code_leading,
              line,
              code_style
            )
          );
        },
      };
    }
  );
}

function value_row(value: string) {
  return {
    height: value_height,
    draw: (page, x: number, y: number) =>
      pdf.text(
        page,
        x + padding,
        y + (value_height + value_style.size) / 2 - 1,
        value,
        value_style
      ),
  };
}

function grids_panel(pair) {
  const input = grid_size(pair.input_matrix);
  const output = grid_size(pair.output_matrix);
  const available: number =
    content_width - 2 * padding - grid_arrow - 2 * panel_gap;
  const cell: number = clamp(
    Math.min(
      available / Math.max(1, input.columns + output.columns),
      grid_height_maximum / Math.max(1, input.rows, output.rows)
    ),
    grid_cell_minimum,
    grid_cell_maximum
  );
  const input_label: string = `input_matrix ${input.rows}×${input.columns}`;
  const output_label: string = `output_matrix ${output.rows}×${output.columns}`;
  const input_width: number = Math.max(
    input.columns * cell,
    pdf.text_width(input_label, label_style)
  );
  const output_width: number = Math.max(
    output.columns * cell,
    pdf.text_width(output_label, label_style)
  );
  const grids: number = Math.max(1, input.rows, output.rows) * cell;
  const output_offset: number = input_width + 2 * panel_gap + grid_arrow;
  return {
    width: output_offset + output_width,
    height: label_height + grids,
    draw: (page, x: number, y: number) => {
      const top: number = y + label_height;
      const baseline: number = y + label_style.size;
      pdf.text(page, x, baseline, input_label, label_style);
      pdf.text(page, x + output_offset, baseline, output_label, label_style);
      draw_grid(page, x, top, pair.input_matrix, cell, true);
      draw_arrow(
        page,
        x + input_width + panel_gap,
        top + grids / 2,
        grid_arrow
      );
      draw_grid(page, x + output_offset, top, pair.output_matrix, cell, true);
    },
  };
}

function draw_implementation(
  page,
  x: number,
  baseline: number,
  line: string,
  name: string
) {
  const bold: string = line.startsWith(name) ? name : "";
  pdf.text(page, x, baseline, bold, name_style);
  pdf.text(
    page,
    x + pdf.text_width(bold, name_style),
    baseline,
    line.slice(bold.length),
    implementation_style
  );
}

function instance_panels(pair, width: number) {
  const inputs = pair.instances.map((instance) =>
    grid_size(instance.input_instance_matrix)
  );
  const outputs = pair.instances.map((instance) =>
    grid_size(instance.output_instance_matrix)
  );
  const extent: number = Math.max(
    1,
    ...[...inputs, ...outputs].flatMap(({ rows, columns }) => [rows, columns])
  );
  const cell: number = clamp(
    thumbnail_extent / extent,
    thumbnail_cell_minimum,
    thumbnail_cell_maximum
  );
  const input_width: number =
    Math.max(0, ...inputs.map(({ columns }) => columns)) * cell;
  const output_width: number =
    Math.max(0, ...outputs.map(({ columns }) => columns)) * cell;
  const arrow_offset: number = padding + input_width + panel_gap;
  const output_offset: number = arrow_offset + thumbnail_arrow + panel_gap;
  const text_offset: number = output_offset + output_width + 2 * panel_gap;
  const columns: number = code_columns(
    width - text_offset - padding,
    implementation_style
  );
  const panels = pair.instances.map((instance, index: number) => {
    const lines = wrap_code(
      instance.implementation,
      columns,
      implementation_hanging
    );
    const thumbnails: number =
      Math.max(1, inputs[index].rows, outputs[index].rows) * cell;
    return {
      height:
        Math.max(thumbnails, lines.length * implementation_leading) +
        2 * instance_padding,
      draw: (page, x: number, y: number) => {
        const top: number = y + instance_padding;
        const baseline: number = top + implementation_style.size;
        draw_grid(
          page,
          x + padding,
          top,
          instance.input_instance_matrix,
          cell,
          false
        );
        draw_arrow(
          page,
          x + arrow_offset,
          top + thumbnails / 2,
          thumbnail_arrow
        );
        draw_grid(
          page,
          x + output_offset,
          top,
          instance.output_instance_matrix,
          cell,
          false
        );
        lines.forEach((line: string, position: number) =>
          draw_implementation(
            page,
            x + text_offset,
            baseline + position * implementation_leading,
            line,
            instance.instance_name
          )
        );
      },
    };
  });
  return { columns, panels };
}

function beside_row(pair, grids) {
  const left: number = grids.width + 2 * padding;
  if (left > content_width * beside_share) {
    return undefined;
  }
  const { columns, panels } = instance_panels(pair, content_width - left);
  const instances: number = sum(panels.map((panel) => panel.height));
  if (
    panels.length === 0 ||
    columns < beside_columns ||
    instances > beside_height_maximum
  ) {
    return undefined;
  }
  const height: number = Math.max(grids.height + 2 * padding, instances);
  return {
    height,
    draw: (page, x: number, y: number, width: number) => {
      grids.draw(page, x + padding, y + padding);
      const separators = [[x + left, y, x + left, y + height]];
      let top: number = y;
      panels.forEach((panel, index: number) => {
        panel.draw(page, x + left, top);
        top += panel.height;
        if (index + 1 < panels.length) {
          separators.push([x + left, top, x + width, top]);
        }
      });
      pdf.lines(page, separators, rule, 0.5);
    },
  };
}

function pair_rows(pair) {
  const grids = grids_panel(pair);
  const beside = beside_row(pair, grids);
  if (beside !== undefined) {
    return [beside];
  }
  return [
    {
      height: grids.height + 2 * padding,
      draw: (page, x: number, y: number) =>
        grids.draw(page, x + padding, y + padding),
    },
    ...instance_panels(pair, content_width).panels,
  ];
}

function draw_box(page, y: number, title: string, aside: string, rows): number {
  const height: number = title_height + sum(rows.map((row) => row.height));
  const baseline: number = y + title_baseline;
  pdf.rects(page, [[margin, y, content_width, title_height]], title_fill);
  pdf.text(page, margin + padding, baseline, title, box_style);
  draw_right(page, content_right - padding, baseline, aside, note_style);
  let top: number = y + title_height;
  const separators = [[margin, top, content_right, top]];
  rows.forEach((row, index: number) => {
    row.draw(page, margin, top, content_width);
    top += row.height;
    if (index + 1 < rows.length) {
      separators.push([margin, top, content_right, top]);
    }
  });
  pdf.lines(page, separators, rule, 0.5);
  pdf.frame(page, [margin, y, content_width, height], rule, 0.6);
  return height;
}

function new_page(layout, section: string): number {
  layout.page = pdf.add_page(layout.doc);
  layout.sections.push(section);
  layout.y = content_top;
  return layout.doc.pages.length - 1;
}

function place_box(layout, title: string, aside: string, rows) {
  const section: string = layout.sections[layout.sections.length - 1];
  const total: number = title_height + sum(rows.map((row) => row.height));
  if (
    layout.y + total > content_bottom &&
    total <= content_bottom - content_top
  ) {
    new_page(layout, section);
  }
  let remaining = rows;
  let heading: string = title;
  while (remaining.length > 0) {
    let used: number = 0;
    let count: number = 0;
    while (
      count < remaining.length &&
      layout.y + title_height + used + remaining[count].height <= content_bottom
    ) {
      used += remaining[count].height;
      count += 1;
    }
    if (count === 0 && layout.y > content_top) {
      new_page(layout, section);
      continue;
    }
    const part = remaining.slice(0, Math.max(1, count));
    layout.y += draw_box(layout.page, layout.y, heading, aside, part);
    layout.y += box_gap;
    remaining = remaining.slice(part.length);
    heading = `${title} (continued)`;
    if (remaining.length > 0) {
      new_page(layout, section);
    }
  }
}

function render_dataset(layout, { file, dataset }): number {
  const page: number = new_page(layout, file);
  pdf.bookmark(layout.doc, file, { page, y: 0 });
  const id: string = puzzle_id(file);
  const summary: string = [
    id === undefined ? "Genesis dataset" : `Puzzle ${id}`,
    plural(dataset.dataset.length, "pair"),
    plural(instance_count(dataset), "instance"),
  ].join("  ·  ");
  pdf.text(layout.page, margin, layout.y + 14, file, heading_style);
  pdf.text(layout.page, margin, layout.y + 28, summary, summary_style);
  layout.y += 40;

  place_box(
    layout,
    "class_declaration",
    "",
    code_rows(dataset.class_declaration)
  );
  place_box(
    layout,
    "declarations",
    plural(dataset.declarations.length, "declaration"),
    dataset.declarations.flatMap((declaration: string, index: number) =>
      code_rows(declaration, String(index))
    )
  );
  place_box(layout, "output_canvas", "", [value_row(dataset.output_canvas)]);
  dataset.dataset.forEach((pair, index: number) =>
    place_box(
      layout,
      `Pair ${index + 1}`,
      plural(pair.instances.length, "instance"),
      pair_rows(pair)
    )
  );
  return page;
}

function draw_swatches(page, y: number) {
  pdf.text(page, margin, y + 10, "Colors", box_style);
  palette.forEach((_, value: number) => {
    const x: number = margin + 40 + value * (swatch_size + swatch_gap);
    const area = [x, y, swatch_size, swatch_size];
    pdf.rects(page, [area], color_of(value));
    pdf.frame(page, area, grid_line, 0.4);
    draw_digit(page, x, y, swatch_size, value);
  });
}

function contents_header(page, y: number): number {
  const baseline: number = y + 10;
  const columns = contents_columns;
  pdf.text(page, columns.dataset, baseline, "Dataset", column_style);
  pdf.text(page, columns.canvas, baseline, "output_canvas", column_style);
  draw_right(page, columns.pairs, baseline, "Pairs", column_style);
  draw_right(page, columns.instances, baseline, "Instances", column_style);
  draw_right(page, columns.page, baseline, "Page", column_style);
  pdf.lines(page, [[margin, y + 14, content_right, y + 14]], rule, 0.6);
  return y + 16;
}

function contents(layout, entries) {
  const first: number = new_page(layout, "Contents");
  pdf.bookmark(layout.doc, "Contents", { page: first, y: 0 });
  const page = layout.page;
  let y: number = content_top;
  pdf.text(page, margin, y + 18, document_title, title_style);
  y += 32;
  for (const paragraph of introduction) {
    for (const line of wrap_text(paragraph, body_style, content_width)) {
      pdf.text(page, margin, y + 9, line, body_style);
      y += 12;
    }
    y += 5;
  }
  const datasets = entries.map(({ dataset }) => dataset);
  const totals: string = [
    plural(datasets.length, "dataset"),
    plural(sum(datasets.map((dataset) => dataset.dataset.length)), "pair"),
    plural(sum(datasets.map(instance_count)), "instance"),
  ].join("  ·  ");
  pdf.text(page, margin, y + 9, totals, total_style);
  y += 22;
  draw_swatches(page, y);
  y = contents_header(page, y + swatch_size + 16);

  const slots = [];
  let current: number = first;
  for (let index: number = 0; index < entries.length; index++) {
    if (y + contents_row > content_bottom) {
      current = new_page(layout, "Contents");
      y = contents_header(layout.page, content_top);
    }
    slots.push({ page: current, y });
    y += contents_row;
  }
  return slots;
}

function fill_contents(layout, entries, slots, starts) {
  const columns = contents_columns;
  entries.forEach(({ file, dataset }, index: number) => {
    const { page: number, y } = slots[index];
    const page = layout.doc.pages[number];
    const baseline: number = y + 9.5;
    const row = [margin, y, content_width, contents_row];
    if (index % 2 === 1) {
      pdf.rects(page, [row], stripe_fill);
    }
    pdf.text(page, columns.dataset, baseline, file, file_style);
    pdf.text(page, columns.canvas, baseline, dataset.output_canvas, file_style);
    const counts = [
      [columns.pairs, dataset.dataset.length],
      [columns.instances, instance_count(dataset)],
      [columns.page, starts[index] + 1],
    ];
    for (const [right, count] of counts) {
      draw_right(page, right, baseline, String(count), count_style);
    }
    pdf.link(page, row, { page: starts[index], y: 0 });
  });
}

function decorate(layout) {
  const total: number = layout.doc.pages.length;
  layout.doc.pages.forEach((page, index: number) => {
    if (index > 0) {
      const section: string = layout.sections[index];
      pdf.text(page, margin, header_baseline, document_title, note_style);
      draw_right(page, content_right, header_baseline, section, note_style);
      pdf.lines(
        page,
        [[margin, header_rule, content_right, header_rule]],
        rule,
        0.5
      );
    }
    const footer: string = `Page ${index + 1} of ${total}`;
    pdf.text(page, margin, footer_baseline, "arc/dataset.pdf", note_style);
    draw_right(page, content_right, footer_baseline, footer, note_style);
  });
}

function main() {
  const entries = load();
  const layout = {
    doc: pdf.create(page_width, page_height, document_title),
    sections: [],
    page: undefined,
    y: content_top,
  };
  const slots = contents(layout, entries);
  const starts = entries.map((entry) => render_dataset(layout, entry));
  fill_contents(layout, entries, slots, starts);
  decorate(layout);
  const bytes = pdf.bytes(layout.doc);
  writeFileSync(output_path, bytes);
  console.log(
    `Wrote ${output_path}: ${plural(layout.doc.pages.length, "page")}, ` +
      `${plural(entries.length, "dataset")}, ${bytes.length} bytes`
  );
}

main();
