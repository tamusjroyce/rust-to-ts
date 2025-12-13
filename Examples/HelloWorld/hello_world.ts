// Converted from Rust: fn main(...)
export function main(): void {
  // Rust macro
  console.log(`Hello, World!`);
  // Rust variable declaration
  const sum = add(2, 2);
  // Rust macro
  console.log(`add(2, 2) = ${sum}`);
  // Rust variable declaration
  const person = { name: "Not Sure", age: 30 };
  // Rust macro
  console.log(`Person: name=${person.name}, age=${person.age}`);
}

// Converted from Rust: fn add(...)
export function add(x: number, y: number): number {
  // Rust expression
  return x + y;
}

// Converted from Rust: struct Person
interface Person {
  name: string;
  age: number;
}

