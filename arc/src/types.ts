export type Grid = number[][];

export interface AnalyzedInstance {
  input_instance_matrix: Grid;
  output_instance_matrix: Grid;
}

export interface TrainingInstance extends AnalyzedInstance {
  instance_name: string;
  input_code: string;
  input_value: unknown;
  output_value: unknown;
}

export interface TrainingExample {
  input_matrix: Grid;
  output_matrix: Grid;
  instances: TrainingInstance[];
}

export interface TrainingInput {
  dataset: TrainingExample[];
}

export interface TrainingDataset extends TrainingInput {
  declarations: string[];
}

export interface TestInstance {
  input_instance_matrix: Grid;
}

export interface Message {
  role: "user" | "assistant";
  content: string;
}

export interface GenerateOptions {
  messages?: Message[];
}
