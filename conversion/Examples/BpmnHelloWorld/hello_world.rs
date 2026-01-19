fn noop() {
    // TODO: implement called function
}

fn main() {
    // Demonstrates standard BPMN -> Rust subset
    let x = 7;
    let mut y = 1;
    y = 2;
    println!("Hello, BPMN!");
    println!("{}", x);
    noop();
    // Skipping BPMN node id="EndEvent_1" (not a task)
}
