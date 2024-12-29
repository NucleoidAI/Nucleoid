declare class $ {
  constructor();

  iof: string;
  private pre: boolean;

  before(): void;
  run(): void;
  graph(): void;
  after(): void;

  get prepared(): boolean;
  set prepared(prepared: boolean);
}

export = $;
