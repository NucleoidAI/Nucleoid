class NODE {
  type: string;
  key: string;
  next: Record<string, any>;
  previous: Record<string, any>;
  sequence: number;

  constructor(key: string);

  before(): void;
  run(scope): NODE | NODE[] | null;
  beforeGraph(): void;
  graph(scope): NODE[] | null;
  after(): void;

  static register(key: string, node: NODE): void;
  static replace(sourceKey: string, targetNode: NODE): void;
  static direct(sourceKey: string, targetKey: string, targetNode: NODE): void;
}

export = NODE;
