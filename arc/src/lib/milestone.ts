import * as Matrix from "./Matrix.ts";
import * as terminal from "./terminal.ts";

function oriented(title: string, orientation: unknown): string {
  const direction: string | undefined = terminal.orientation(orientation);
  return direction ? `${title} · ${direction}` : title;
}

// Adds a drawing's epoch, the column and row of position (0, 0) in its grid,
// when the drawing grew left of or above that position.
function epoched(title: string, epoch): string {
  return epoch && (epoch.x !== 0 || epoch.y !== 0)
    ? `${title} · epoch x=${epoch.x} y=${epoch.y}`
    : title;
}

function signed(value: number): string {
  return value < 0 ? `${value}` : `+${value}`;
}

export function start({
  version,
  puzzle_id,
  only_puzzle_dataset,
  train,
  test_input_matrix,
}) {
  terminal.header({ version, puzzle_id, only_puzzle_dataset });
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
  instances.forEach((instance) => {
    terminal.grid("Input", instance.input_instance_matrix, {
      key: "training_input",
      loop: true,
    });
    terminal.grid("Output", instance.output_instance_matrix, {
      key: "training_output",
      loop: true,
    });
  });
}

export function training_objects_synthesized({ instances }) {
  terminal.step(`${instances.length} training objects synthesized`, {
    key: "training_objects_synthesized",
    loop: true,
  });
  instances.forEach((instance) => {
    const { source, rotation, offset } = instance.synthesis;
    const title: string = `${source} · ${signed(rotation)}° x${signed(
      offset.x
    )} y${signed(offset.y)}`;
    terminal.grid(
      epoched(`Input · ${title}`, instance.input_instance_epoch),
      instance.input_instance_matrix,
      { key: "synthesized_input", loop: true }
    );
    terminal.grid(
      epoched(`Output · ${title}`, instance.output_instance_epoch),
      instance.output_instance_matrix,
      { key: "synthesized_output", loop: true }
    );
  });
}

export function constructing_instance({ instance_name }) {
  terminal.step(`Constructing ${instance_name}`, {
    key: "constructing_instance",
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
  terminal.grid(
    `Input crop · ${instance_name}`,
    Matrix.crop(input_instance_matrix),
    { key: "input_crop", loop: true }
  );
  terminal.grid(
    `Output crop · ${instance_name}`,
    Matrix.crop(output_instance_matrix),
    { key: "output_crop", loop: true }
  );
  terminal.code(`Implementation · ${instance_name}`, implementation, {
    key: "implementation",
    loop: true,
  });
  terminal.value(
    oriented(`Input state · ${instance_name}`, input_value?.orientation),
    input_value,
    { key: "input_state", loop: true }
  );
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
  instances.forEach(({ input_instance_matrix }) => {
    terminal.grid("Input", input_instance_matrix, {
      key: "test_input_object",
      loop: true,
    });
  });
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
  terminal.grid(
    `Input crop · ${instance_name}`,
    Matrix.crop(input_instance_matrix),
    { key: "input_crop", loop: true }
  );
  terminal.code(`Implementation · ${instance_name}`, implementation, {
    key: "implementation",
    loop: true,
  });
}

export function executing_transformation({ instance_name }) {
  terminal.step(`Executing the object transformation · ${instance_name}`, {
    key: "executing_transformation",
    loop: true,
  });
}

export function transformation_executed({ instance_name, input_value }) {
  terminal.value(
    oriented(`Input state · ${instance_name}`, input_value?.orientation),
    input_value,
    { key: "input_state", loop: true }
  );
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

export function output_placed({ instance_name, output_instance_drawing }) {
  const { matrix, epoch } = output_instance_drawing;
  terminal.grid(epoched(`Placed output · ${instance_name}`, epoch), matrix, {
    key: "placed_output",
    loop: true,
  });
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
  terminal.failure(error);
}
