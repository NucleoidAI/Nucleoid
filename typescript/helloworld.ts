import nucleoid from "./";

export class Item {
  public name: string;
  public barcode: string;

  constructor(name: string, barcode: string) {
    this.name = name;
    this.barcode = barcode;
  }

  static find(predicate: (item: Item) => boolean): Item | undefined {
    const items: Item[] = [];
    return items.find(predicate);
  }
}

nucleoid.register(Item);

const app = nucleoid.express({});

// 👍 Only needed a business logic and 💖
// "Create an item with given name and barcode,
// but the barcode must be unique"
app.post("/items", (req) => {
  const { name, barcode } = req.body;

  const existing = Item.find((i) => i.barcode === barcode);
  if (existing) {
    throw new Error("DUPLICATE_BARCODE");
  }

  return new Item(name, barcode);
});

app.listen(3000, () => {
  console.log("🚀 Server listening on http://localhost:3000");
});
