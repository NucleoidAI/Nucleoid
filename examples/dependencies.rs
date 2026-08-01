//! Shows what makes Nucleoid declarative: statements are kept, not just run.
//!
//! Run with `cargo run --example dependencies`.

use nucleoid::Runtime;

fn main() {
    let mut runtime = Runtime::new();

    // An assignment states a relationship that the runtime maintains.
    runtime.run("celsius = 100").unwrap();
    runtime.run("fahrenheit = celsius * 9 / 5 + 32").unwrap();

    println!("celsius 100 -> {}", read(&mut runtime, "fahrenheit"));

    // Changing what it reads brings the relationship up to date.
    runtime.run("celsius = 37").unwrap();
    println!("celsius  37 -> {}", read(&mut runtime, "fahrenheit"));

    // The same holds for types and their instances.
    runtime
        .run(
            "class Sensor(name: str):\n    this.name = name\n\
             \n\
             $Sensor.label = \"sensor:\" + $Sensor.name\n",
        )
        .unwrap();

    runtime.run("kitchen = Sensor(\"kitchen\")").unwrap();
    println!("label      -> {}", read(&mut runtime, "kitchen.label"));

    // A rule declared on the type applies to instances made later, too.
    runtime.run("hallway = Sensor(\"hallway\")").unwrap();
    println!("label      -> {}", read(&mut runtime, "hallway.label"));
}

fn read(runtime: &mut Runtime, source: &str) -> String {
    runtime
        .run(source)
        .map(|value| value.to_string())
        .unwrap_or_else(|error| error.to_string())
}
