interface Person {
  name: string;
  age: number;
}

export function main(): void {
  console.log(`Hello, World!`);
  const sum = add(2, 2);
  console.log(`add(2, 2) = ${sum}`);
  const person = { name: "Not Sure", age: 30 };
  console.log(`Person: name=${person.name}, age=${person.age}`);
}

export function add(x: number, y: number): number {
  return x + y;
}
