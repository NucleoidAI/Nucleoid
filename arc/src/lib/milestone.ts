import { inspect } from "node:util";
import * as Matrix from "./Matrix.ts";
import * as terminal from "./terminal.ts";

function oriented(title: string, orientation: unknown): string {
  const direction: string | undefined = terminal.orientation(orientation);
  return direction ? `${title} · ${direction}` : title;
}

let synthesized:
  | { instance_name: string; panels: terminal.Panel[] }
  | undefined;

export function start({ version, puzzle_id, train, test_input_matrix }) {
  synthesized = undefined;
  terminal.header({ version, puzzle_id });
  terminal.puzzle({ train, testInput: test_input_matrix });
}

export function analysis() {
  terminal.section(
    1,
    "Analyze",
    "Infer reusable objects and transformation rules"
  );
}

export function inferring_declarations() {
  terminal.step("Inferring declarations");
}

export function declarations_inferred({ declarations, output_canvas }) {
  terminal.code(`Declarations · Output canvas ${output_canvas}`, declarations);
}

export function segmenting_training_objects() {
  terminal.step("Segmenting training objects", {
    key: "segmenting_training_objects",
    loop: true,
  });
}

export function training_objects_segmented({ instances }) {
  terminal.panels(
    "",
    instances.map(({ input_instance_matrix, output_instance_matrix }) => [
      terminal.gridPanel("Input", input_instance_matrix, {
        key: "training_input",
        loop: true,
      }),
      terminal.gridPanel("Output", output_instance_matrix, {
        key: "training_output",
        loop: true,
      }),
    ])
  );
}

export function synthesizing_training_dataset() {
  terminal.step("Synthesizing training pairs");
}

export function synthesized_instance_rejected({
  instance_name,
  output_value,
  output_measurement,
}) {
  const mismatches: string[] = Object.entries(output_measurement).flatMap(
    ([property, measured]) => {
      const state: unknown = output_value?.[property];
      return state === measured
        ? []
        : [`${property} ${inspect(state)} ≠ ${inspect(measured)}`];
    }
  );
  terminal.rejection(
    [
      `Rejected ${instance_name}`,
      "Output state ≠ Placed output",
      ...mismatches,
    ].join(" · "),
    { key: "synthesized_instance_rejected", loop: true }
  );
}

export function training_dataset_synthesized({ dataset }) {
  const pairs: string = dataset.length === 1 ? "pair" : "pairs";
  terminal.step(`${dataset.length} training ${pairs} synthesized`);
  dataset.forEach(({ instances }, index: number) => {
    for (const {
      instance_name,
      input_instance_matrix,
      output_instance_matrix,
      output_value,
    } of instances) {
      terminal.panels(`Synthesized ${index + 1} · ${instance_name}`, [
        [
          terminal.gridPanel("Input", input_instance_matrix, {
            key: "synthesized_input",
            loop: true,
          }),
          terminal.gridPanel("Output", output_instance_matrix, {
            key: "synthesized_output",
            loop: true,
          }),
          terminal.valuePanel(
            oriented("Output state", output_value?.orientation),
            output_value,
            { key: "synthesized_output_state", loop: true }
          ),
        ],
      ]);
    }
  });
}

export function constructing_instance({ instance_name }) {
  terminal.step(`Constructing ${instance_name}`, {
    key: "constructing_instance",
    loop: true,
  });
}

export function deriving_instance({ instance_name, source }) {
  terminal.step(`Deriving ${instance_name} from ${source}`, {
    key: "deriving_instance",
    loop: true,
  });
}

export function evaluating_rules({ instance_name }) {
  terminal.step(`Evaluating declarative rules · ${instance_name}`, {
    key: "evaluating_rules",
    loop: true,
  });
}

export function instance_constructed({
  instance_name,
  implementation,
  input_instance_matrix,
  output_instance_matrix,
  input_value,
}) {
  terminal.panels(instance_name, [
    [
      terminal.gridPanel("Input crop", Matrix.crop(input_instance_matrix), {
        key: "input_crop",
        loop: true,
      }),
      terminal.gridPanel("Output crop", Matrix.crop(output_instance_matrix), {
        key: "output_crop",
        loop: true,
      }),
      terminal.codePanel("Implementation", implementation, {
        key: "implementation",
        loop: true,
      }),
      terminal.valuePanel(
        oriented("Input state", input_value?.orientation),
        input_value,
        { key: "input_state", loop: true }
      ),
    ],
  ]);
}

export function state({
  phase,
  obj_list,
}: {
  phase: "analysis" | "visualization";
  obj_list: Record<string, unknown>[];
}) {
  const properties = [
    ...new Set(obj_list.flatMap((instance) => Object.keys(instance))),
  ].filter((property: string) => property !== "id");

  terminal.table(
    phase === "analysis" ? "Analysis · State" : "Visualization · State",
    ["id", ...properties],
    obj_list,
    { key: "state" }
  );
}

export function visualization() {
  terminal.section(
    2,
    "Visualize",
    "Apply the learned world model to the test input"
  );
}

export function segmenting_test_input({ test_input_matrix }) {
  terminal.step("Segmenting the test input");
  terminal.grid("Test input", test_input_matrix);
}

export function test_input_segmented({ instances }) {
  terminal.step(`${instances.length} objects identified`);
  terminal.panels("", [
    instances.map(({ input_instance_matrix }) =>
      terminal.gridPanel("Input", input_instance_matrix, {
        key: "test_input_object",
        loop: true,
      })
    ),
  ]);
}

export function synthesizing_instance({ instance_name }) {
  terminal.step(`Synthesizing ${instance_name}`, {
    key: "synthesizing_instance",
    loop: true,
  });
}

export function instance_synthesized({
  instance_name,
  input_instance_matrix,
  implementation,
}) {
  synthesized = {
    instance_name,
    panels: [
      terminal.gridPanel("Input crop", Matrix.crop(input_instance_matrix), {
        key: "input_crop",
        loop: true,
      }),
      terminal.codePanel("Implementation", implementation, {
        key: "implementation",
        loop: true,
      }),
    ],
  };
}

function flush_synthesized() {
  if (synthesized) {
    terminal.panels(synthesized.instance_name, [synthesized.panels]);
    synthesized = undefined;
  }
}

export function executing_transformation({ instance_name }) {
  terminal.step(`Executing the object transformation · ${instance_name}`, {
    key: "executing_transformation",
    loop: true,
  });
}

export function transformation_executed({ instance_name, input_value }) {
  if (synthesized?.instance_name !== instance_name) {
    flush_synthesized();
  }
  const panels: terminal.Panel[] = synthesized?.panels ?? [];
  synthesized = undefined;
  terminal.panels(instance_name, [
    [
      ...panels,
      terminal.valuePanel(
        oriented("Input state", input_value?.orientation),
        input_value,
        { key: "input_state", loop: true }
      ),
    ],
  ]);
}

export function composing_output({ instance_name }) {
  terminal.step(`Composing the output object · ${instance_name}`, {
    key: "composing_output",
    loop: true,
  });
}

export function output_composed({
  instance_name,
  input_instance_matrix,
  output_instance_matrix,
  output_value,
}) {
  terminal.grid(
    `Input crop · ${instance_name}`,
    Matrix.crop(input_instance_matrix),
    { key: "composition_input_crop", loop: true }
  );
  terminal.grid(
    oriented(`Output crop · ${instance_name}`, output_value.orientation),
    output_instance_matrix,
    { key: "output_crop", loop: true }
  );
  terminal.value(
    oriented(`Output state · ${instance_name}`, output_value.orientation),
    output_value,
    { key: "output_state", loop: true }
  );
}

export function output_placed({
  instance_name,
  output_instance_padded_matrix,
}) {
  terminal.grid(
    `Placed output · ${instance_name}`,
    output_instance_padded_matrix,
    { key: "placed_output", loop: true }
  );
}

export function comparison({ test_output_matrix, result_matrix }) {
  terminal.section(3, "Compare", "Expected and predicted canvases");
  terminal.grid("Expected", test_output_matrix);
  terminal.grid("Result", result_matrix);
}

export function summary({ difference, duration }) {
  terminal.summary(difference, duration);
}

export function usage({ prompt_tokens, completion_tokens, cached_tokens }) {
  terminal.usage({ prompt_tokens, completion_tokens, cached_tokens });
}

export function prompt_cache_alert({ cached_tokens }) {
  terminal.alert(
    typeof cached_tokens === "number"
      ? `Provider prompt cache used · ${cached_tokens} cached input tokens`
      : "Provider prompt cache unverified · no cached token count reported"
  );
}

export function failure(error) {
  flush_synthesized();
  terminal.failure(error);
}
