fn main() {
    println!("Hello, World!");

    // Use add()
    let sum = add(2, 2);
    println!("add(2, 2) = {}", sum);

    // Allocate and use Person
    let person = Person {
        name: String::from("Not Sure"),
        age: 30,
    };
    println!("Person: name={}, age={}", person.name, person.age);
}

fn add(x: i32, y: i32) -> i32 {
    return x + y;
}

struct Person {
    name: String,
    age: u32,
}
