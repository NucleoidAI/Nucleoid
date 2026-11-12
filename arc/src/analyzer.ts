import * as nucleoid from "./lib/nucleoid";
import * as llm from "./lib/llm";
import * as milestone from "./lib/milestone.ts";
import * as Matrix from "./lib/Matrix.ts";

export async function declarations({
  puzzle_id,
  only_puzzle_dataset,
  train_dataset,
}) {
  milestone.inferring_declarations();

  const { declarations, output_canvas } = await llm.generate({
    puzzle_id,
    only_puzzle_dataset,
    grammar: true,
    messages: [
      {
        role: "user",
        content: `
          query:
          ${JSON.stringify(train_dataset)}
          json_format:
          { "declarations": <DECLARATIONS>, "output_canvas": <OUTPUT_CANVAS> }
        `,
      },
    ],
  });

  milestone.declarations_inferred({ declarations, output_canvas });

  return { declarations, output_canvas };
}

export async function instances({
  puzzle_id,
  only_puzzle_dataset,
  train_dataset,
}) {
  milestone.segmenting_training_objects();

  const { dataset } = await llm.generate({
    puzzle_id,
    only_puzzle_dataset,
    messages: [
      {
        role: "user",
        content: `
          query:
          ${JSON.stringify(train_dataset)}
          json_format:
          {
            "dataset": [{
              "instances": [{
                "input_instance_matrix": <INPUT_INSTANCE_MATRIX>,
                "output_instance_matrix": <OUTPUT_INSTANCE_MATRIX>
              }]
            }]
          }
        `,
      },
    ],
  });

  milestone.training_objects_segmented({ instances: dataset[0].instances });

  return { instances: dataset[0].instances };
}

// Each synthesis turns the extracted instances of a training pair clockwise by
// its rotation and moves them by its offset, so every training pair gains three
// synthesized pairs.
const SYNTHESES = [
  { rotation: 90, offset: { x: 1, y: 1 } },
  { rotation: 90, offset: { x: -1, y: 1 } },
  { rotation: 90, offset: { x: -1, y: -1 } },
];

// Draws the object of an instance matrix turned clockwise by the rotation about
// the pivot, the bounding box the instance turns about, which keeps its top-left
// corner, and moved by the offset, on a canvas of the matrix's size that grows
// as needed. The gaps between the object's and the pivot's edges turn with
// them, so the object keeps its place relative to the pivot. An object without
// cells stays an all-zero canvas.
function turn(matrix, pivot, rotation: number, offset) {
  const object = Matrix.locate(matrix);
  const rows: number = matrix.length;
  const cols: number = matrix[0]?.length ?? 0;
  if (object.width === 0) {
    return Matrix.draw([], rows, cols, 0, 0);
  }

  const turned = Matrix.rotate(object.matrix, rotation);
  const left: number = object.x - pivot.x;
  const top: number = object.y - pivot.y;
  const right: number = pivot.x + pivot.width - (object.x + object.width);
  const bottom: number = pivot.y + pivot.height - (object.y + object.height);
  const [x, y] = [
    [left, top],
    [bottom, left],
    [right, bottom],
    [top, right],
  ][(((rotation / 90) % 4) + 4) % 4];

  return Matrix.draw(
    turned,
    rows,
    cols,
    pivot.x + offset.x + x,
    pivot.y + offset.y + y
  );
}

// Draws the drawing on the frame that covers every drawing, so their grids line
// up, and returns its grid, which starts at the frame's top-left cell.
function frame(drawing, drawings) {
  const blanks = drawings.map((other) => ({
    ...other,
    matrix: other.matrix.map((row) => row.map(() => 0)),
  }));
  return Matrix.merge(drawing, ...blanks).matrix;
}

// The training pair that one synthesis makes of a training pair's extracted
// instances: each instance's input and output objects turn together about its
// input object's bounding box, or its output object's when the input is empty,
// and move by the offset, so each keeps its place relative to the other. Every
// grid of the pair shares one frame, which covers the instances' canvas and
// every turned object and starts at its top-left cell, so the pair's positions
// are never negative. The pair's input and output matrices merge its
// instances' grids, the output under the output canvas, so a later object wins
// where turned objects overlap.
function synthesized_pair(instances, { rotation, offset }, output_canvas) {
  const drawings = instances.map((instance) => {
    const input = Matrix.locate(instance.input_instance_matrix);
    const pivot =
      input.width > 0 ? input : Matrix.locate(instance.output_instance_matrix);

    return {
      source: instance.instance_name,
      input: turn(instance.input_instance_matrix, pivot, rotation, offset),
      output: turn(instance.output_instance_matrix, pivot, rotation, offset),
    };
  });
  const drawn = drawings.flatMap(({ input, output }) => [input, output]);
  const synthesized = drawings.map(({ source, input, output }) => ({
    input_instance_matrix: frame(input, drawn),
    output_instance_matrix: frame(output, drawn),
    synthesis: { source, rotation, offset: { ...offset } },
  }));

  const input_matrix = Matrix.merge(
    ...synthesized.map(({ input_instance_matrix }) => input_instance_matrix)
  ).matrix;
  const output_matrix = Matrix.merge(
    ...synthesized.map(({ output_instance_matrix }) => output_instance_matrix)
  ).matrix;

  return {
    input_matrix,
    output_matrix: Matrix.canvas(output_matrix, output_canvas),
    instances: synthesized,
  };
}

// Synthesizes three training pairs from a training pair's extracted instances,
// one per synthesis, each holding every instance turned and moved by it. A
// pair without instances synthesizes none.
export function synthesize({ instances, output_canvas }) {
  const dataset =
    instances.length > 0
      ? SYNTHESES.map((synthesis) =>
          synthesized_pair(instances, synthesis, output_canvas)
        )
      : [];

  milestone.training_objects_synthesized({ dataset });

  return { dataset };
}

export async function value({
  puzzle_id,
  only_puzzle_dataset,
  train_dataset,
  input_matrix,
  instance,
}) {
  milestone.constructing_instance({ instance_name: instance.instance_name });

  const { x, y, width, height } = Matrix.locate(instance.input_instance_matrix);

  const { orientation } = await llm.generate({
    puzzle_id,
    only_puzzle_dataset,
    few_shot: (train_dataset) =>
      train_dataset.dataset
        .flat()
        .flatMap(({ instances }) => instances)
        .map(({ input_instance_matrix, input_value }) => ({
          matrix: Matrix.silhouette(Matrix.crop(input_instance_matrix)),
          orientation: input_value.orientation,
        })),
    messages: [
      {
        role: "assistant",
        content: `
          current:
          ${JSON.stringify(
            train_dataset.dataset
              .flat()
              .filter(({ instances }) => instances)
              .flatMap(({ instances }) => instances)
              .filter(({ input_value }) => input_value)
              .map(({ input_instance_matrix, input_value }) => ({
                matrix: Matrix.silhouette(Matrix.crop(input_instance_matrix)),
                orientation: input_value.orientation,
              }))
          )}
        `,
      },
      {
        role: "user",
        content: `
          query:
          ${JSON.stringify({
            matrix: Matrix.silhouette(
              Matrix.crop(instance.input_instance_matrix)
            ),
          })}
          json_format:
          {
            "orientation": <ORIENTATION>,
          }
        `,
      },
    ],
  });

  const { dataset } = await llm.generate({
    puzzle_id,
    only_puzzle_dataset,
    few_shot: (train_dataset) => ({
      ...train_dataset,
      declarations: train_dataset.declarations[0],
      dataset: train_dataset.dataset.map(({ instances }) => ({
        instances: instances.map((instance) => ({
          ...instance,
          input_instance_matrix: Matrix.crop(instance.input_instance_matrix),
          output_instance_matrix: Matrix.crop(instance.output_instance_matrix),
        })),
      })),
    }),
    messages: [
      {
        role: "assistant",
        content: `
          current:
          ${JSON.stringify({
            ...train_dataset,
            declarations: train_dataset.declarations[0],
            dataset: train_dataset.dataset
              .filter(({ instances }) => instances)
              .map(({ instances }) => ({
                instances: instances
                  .filter(({ implementation }) => implementation)
                  .map((instance) => ({
                    ...instance,
                    input_matrix: Matrix.crop(instance.input_instance_matrix),
                    output_matrix: Matrix.crop(instance.output_instance_matrix),
                  })),
              })),
          })}
        `,
      },
      {
        role: "user",
        content: `
          query:
          ${JSON.stringify({
            dataset: [
              {
                instances: [instance],
              },
            ],
          })}
          input_x: ${x}
          input_y: ${y}
          input_width: ${width}
          input_height: ${height}
          orientation: ${orientation}
          json_format:
          {
            "dataset": [{
              "instances": [{
                "implementation": <IMPLEMENTATION>
              }]
            }]
          }  
        `,
      },
    ],
  });

  const implementation = dataset[0].instances[0].implementation;

  milestone.evaluating_rules({ instance_name: instance.instance_name });

  const canvas_width: number = input_matrix[0].length;
  const canvas_height: number = input_matrix.length;
  const input_value = await nucleoid.run(
    [
      `canvas_width = ${canvas_width}`,
      `canvas_height = ${canvas_height}`,
      train_dataset.declarations[0],
      implementation,
    ].join("\n\n")
  );

  milestone.instance_constructed({
    instance_name: instance.instance_name,
    input_instance_matrix: instance.input_instance_matrix,
    output_instance_matrix: instance.output_instance_matrix,
    implementation,
    input_value,
  });

  return { implementation, input_value };
}

export async function state(train_dataset) {
  const objects = [];

  for (const dataset of train_dataset.dataset) {
    const implementations = dataset.instances.map(
      ({ implementation }) => implementation
    );

    const canvas_width: number = dataset.input_matrix[0].length;
    const canvas_height: number = dataset.input_matrix.length;
    const obj_list = await nucleoid.run(
      [
        `canvas_width = ${canvas_width}`,
        `canvas_height = ${canvas_height}`,
        ...train_dataset.declarations,
        ...implementations,
        "Obj[:]",
      ].join("\n\n")
    );

    objects.push(...obj_list);

    [...obj_list].forEach((instance) => {
      const found_instance = dataset.instances.find(
        ({ instance_name }) => instance.id === instance_name
      );
      found_instance.output_value = instance;
    });
  }

  milestone.state({ phase: "analysis", obj_list: objects });
}
