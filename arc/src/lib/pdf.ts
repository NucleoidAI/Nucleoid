import { deflateSync } from "node:zlib";

const fonts = ["sans", "sans_bold", "mono", "mono_bold"];

const base_fonts = {
  sans: "Helvetica",
  sans_bold: "Helvetica-Bold",
  mono: "Courier",
  mono_bold: "Courier-Bold",
};

// Advance widths of ASCII 32 to 126 in thousandths of the font size, from the
// Adobe metrics of the standard Helvetica fonts; Courier advances 600.
const sans_widths = [
  278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278,
  278, 556, 556, 556, 556, 556, 556, 556, 556, 556, 556, 278, 278, 584, 584,
  584, 556, 1015, 667, 667, 722, 722, 667, 611, 778, 722, 278, 500, 667, 556,
  833, 722, 778, 667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611, 278,
  278, 278, 469, 556, 333, 556, 556, 500, 556, 556, 278, 556, 556, 222, 222,
  500, 222, 833, 556, 556, 556, 556, 333, 500, 278, 556, 500, 722, 500, 500,
  500, 334, 260, 334, 584,
];

const sans_bold_widths = [
  278, 333, 474, 556, 556, 889, 722, 238, 333, 333, 389, 584, 278, 333, 278,
  278, 556, 556, 556, 556, 556, 556, 556, 556, 556, 556, 333, 333, 584, 584,
  584, 611, 975, 722, 722, 722, 722, 667, 611, 778, 722, 278, 556, 722, 611,
  833, 722, 778, 667, 778, 722, 667, 611, 722, 667, 944, 667, 667, 611, 333,
  278, 333, 584, 556, 333, 556, 611, 556, 611, 556, 333, 611, 611, 278, 278,
  556, 278, 889, 611, 611, 611, 611, 389, 556, 333, 611, 556, 778, 556, 556,
  500, 389, 280, 389, 584,
];

const mono_width: number = 600;

const win_ansi = new Map([
  ["•", 0x95],
  ["–", 0x96],
  ["—", 0x97],
  ["·", 0xb7],
  ["×", 0xd7],
]);

const win_ansi_widths = new Map([
  [0x95, 350],
  [0x96, 556],
  [0x97, 1000],
  [0xb7, 278],
  [0xd7, 584],
]);

const unknown_code: number = 0x3f;

function codes(text: string) {
  return Array.from(text, (character: string) => {
    const code: number = character.codePointAt(0) ?? unknown_code;
    return code >= 32 && code <= 126
      ? code
      : win_ansi.get(character) ?? unknown_code;
  });
}

function advance(code: number, font: string): number {
  if (font === "mono" || font === "mono_bold") {
    return mono_width;
  }
  const widths = font === "sans_bold" ? sans_bold_widths : sans_widths;
  return widths[code - 32] ?? win_ansi_widths.get(code) ?? mono_width;
}

export function text_width(text: string, { font, size }): number {
  const units: number = codes(text).reduce(
    (total: number, code: number) => total + advance(code, font),
    0
  );
  return (units * size) / 1000;
}

function literal(text: string): string {
  return codes(text)
    .map((code: number) => {
      if (code === 0x28 || code === 0x29 || code === 0x5c) {
        return `\\${String.fromCharCode(code)}`;
      }
      return code > 126
        ? `\\${code.toString(8).padStart(3, "0")}`
        : String.fromCharCode(code);
    })
    .join("");
}

function format(value: number): string {
  const rounded: number = Math.round(value * 100) / 100;
  return Object.is(rounded, -0) ? "0" : String(rounded);
}

function channels(color): string {
  return color
    .map((channel: number) => String(Math.round((channel / 255) * 1000) / 1000))
    .join(" ");
}

// A document of pages drawn with the origin at the top left and y growing
// downward, in the standard Helvetica and Courier fonts. It writes no dates
// or random identifiers, so equal content gives equal bytes.
export function create(width: number, height: number, title: string) {
  return { width, height, title, pages: [], bookmarks: [] };
}

export function add_page(doc) {
  const page = {
    height: doc.height,
    operations: [],
    links: [],
    fill: "",
    stroke: "",
    line_width: "",
  };
  doc.pages.push(page);
  return page;
}

export function bookmark(doc, title: string, target) {
  doc.bookmarks.push({ title, target });
}

export function link(page, area, target) {
  page.links.push({ area, target });
}

function point(page, x: number, y: number): string {
  return `${format(x)} ${format(page.height - y)}`;
}

function rectangle(page, [x, y, width, height]): string {
  return `${point(page, x, y + height)} ${format(width)} ${format(height)} re`;
}

function fill(page, color) {
  const value: string = channels(color);
  if (value !== page.fill) {
    page.fill = value;
    page.operations.push(`${value} rg`);
  }
}

function stroke(page, color, width: number) {
  const value: string = channels(color);
  if (value !== page.stroke) {
    page.stroke = value;
    page.operations.push(`${value} RG`);
  }
  const line: string = format(width);
  if (line !== page.line_width) {
    page.line_width = line;
    page.operations.push(`${line} w`);
  }
}

export function rects(page, areas, color) {
  if (areas.length > 0) {
    fill(page, color);
    const path: string = areas.map((area) => rectangle(page, area)).join(" ");
    page.operations.push(`${path} f`);
  }
}

export function frame(page, area, color, line_width: number) {
  stroke(page, color, line_width);
  page.operations.push(`${rectangle(page, area)} S`);
}

export function lines(page, segments, color, line_width: number) {
  if (segments.length > 0) {
    stroke(page, color, line_width);
    const path: string = segments
      .map(
        ([x1, y1, x2, y2]) =>
          `${point(page, x1, y1)} m ${point(page, x2, y2)} l`
      )
      .join(" ");
    page.operations.push(`${path} S`);
  }
}

export function polygon(page, points, color) {
  fill(page, color);
  const path: string = points
    .map(
      ([x, y], index: number) =>
        `${point(page, x, y)} ${index === 0 ? "m" : "l"}`
    )
    .join(" ");
  page.operations.push(`${path} h f`);
}

export function text(
  page,
  x: number,
  baseline: number,
  content: string,
  { font, size, color }
) {
  if (content !== "") {
    fill(page, color);
    page.operations.push(
      `BT /F${fonts.indexOf(font) + 1} ${format(size)} Tf ${point(
        page,
        x,
        baseline
      )} Td (${literal(content)}) Tj ET`
    );
  }
}

function link_object(
  doc,
  { area: [x, y, width, height], target },
  destination
): string {
  return `<< /Type /Annot /Subtype /Link /Rect [${format(x)} ${format(
    doc.height - y - height
  )} ${format(x + width)} ${format(
    doc.height - y
  )}] /Border [0 0 0] /Dest ${destination(target)} >>`;
}

function serialize(bodies, count: number) {
  const chunks = [Buffer.from("%PDF-1.4\n%\xe2\xe3\xcf\xd3\n", "latin1")];
  const offsets = [];
  let offset: number = chunks[0].length;
  for (let object: number = 1; object < count; object++) {
    const chunk = Buffer.concat([
      Buffer.from(`${object} 0 obj\n`, "latin1"),
      bodies.get(object),
      Buffer.from("\nendobj\n", "latin1"),
    ]);
    offsets.push(offset);
    chunks.push(chunk);
    offset += chunk.length;
  }
  const xref: string = [
    "xref",
    `0 ${count}`,
    "0000000000 65535 f ",
    ...offsets.map(
      (position: number) => `${String(position).padStart(10, "0")} 00000 n `
    ),
  ].join("\n");
  chunks.push(
    Buffer.from(
      `${xref}\ntrailer\n<< /Size ${count} /Root 1 0 R /Info 3 0 R >>\nstartxref\n${offset}\n%%EOF\n`,
      "latin1"
    )
  );
  return Buffer.concat(chunks);
}

export function bytes(doc) {
  const first_font: number = 4;
  let next: number = first_font + fonts.length;
  const numbers = doc.pages.map((page) => ({
    page: next++,
    content: next++,
    links: page.links.map(() => next++),
  }));
  const outline: number = next++;
  const items = doc.bookmarks.map(() => next++);

  const bodies = new Map();
  const put = (object: number, body: string) =>
    bodies.set(object, Buffer.from(body, "latin1"));
  const reference = (object: number): string => `${object} 0 R`;
  const destination = ({ page, y }): string =>
    `[${reference(numbers[page].page)} /XYZ 0 ${format(doc.height - y)} 0]`;

  put(
    1,
    `<< /Type /Catalog /Pages 2 0 R /Outlines ${reference(
      outline
    )} /PageMode /UseOutlines >>`
  );
  put(
    2,
    `<< /Type /Pages /Kids [${numbers
      .map(({ page }) => reference(page))
      .join(" ")}] /Count ${numbers.length} >>`
  );
  put(3, `<< /Title (${literal(doc.title)}) /Creator (Nucleoid ARC) >>`);
  fonts.forEach((font: string, index: number) =>
    put(
      first_font + index,
      `<< /Type /Font /Subtype /Type1 /BaseFont /${base_fonts[font]} /Encoding /WinAnsiEncoding >>`
    )
  );
  const resources: string = `<< /Font << ${fonts
    .map(
      (_, index: number) => `/F${index + 1} ${reference(first_font + index)}`
    )
    .join(" ")} >> >>`;

  doc.pages.forEach((page, index: number) => {
    const { page: object, content, links } = numbers[index];
    const annotations: string =
      links.length > 0 ? ` /Annots [${links.map(reference).join(" ")}]` : "";
    put(
      object,
      `<< /Type /Page /Parent 2 0 R /MediaBox [0 0 ${format(
        doc.width
      )} ${format(doc.height)}] /Resources ${resources} /Contents ${reference(
        content
      )}${annotations} >>`
    );
    const stream = deflateSync(
      Buffer.from(page.operations.join("\n"), "latin1")
    );
    bodies.set(
      content,
      Buffer.concat([
        Buffer.from(
          `<< /Length ${stream.length} /Filter /FlateDecode >>\nstream\n`,
          "latin1"
        ),
        stream,
        Buffer.from("\nendstream", "latin1"),
      ])
    );
    page.links.forEach((entry, position: number) =>
      put(links[position], link_object(doc, entry, destination))
    );
  });

  put(
    outline,
    items.length === 0
      ? "<< /Type /Outlines /Count 0 >>"
      : `<< /Type /Outlines /First ${reference(items[0])} /Last ${reference(
          items[items.length - 1]
        )} /Count ${items.length} >>`
  );
  doc.bookmarks.forEach(({ title, target }, index: number) => {
    const previous: string =
      index > 0 ? ` /Prev ${reference(items[index - 1])}` : "";
    const following: string =
      index + 1 < items.length ? ` /Next ${reference(items[index + 1])}` : "";
    put(
      items[index],
      `<< /Title (${literal(title)}) /Parent ${reference(
        outline
      )}${previous}${following} /Dest ${destination(target)} >>`
    );
  });
  return serialize(bodies, next);
}
