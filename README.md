# Rust to TypeScript Converter using SWC

This project is a Rust application that converts Rust code into TypeScript using the SWC crate. It leverages the abstract syntax tree (AST) representations of Rust code to generate equivalent TypeScript code, including comments that indicate the original Rust lines.

## Project Structure

- `src/main.rs`: Entry point of the application. Initializes the converter and sets up necessary components.
- `src/ast/mod.rs`: Defines the AST structures for Rust code, including types and functions for parsing and manipulating the Rust AST.
- `src/converter/mod.rs`: Contains the logic for converting Rust AST nodes to TypeScript AST nodes. It traverses the Rust AST and generates TypeScript code with comments indicating the source Rust lines.
- `src/utils/mod.rs`: Provides utility functions for formatting and error handling during the conversion process.
- `Cargo.toml`: Configuration file for the Rust project, specifying metadata, dependencies (including SWC), and build settings.

## Getting Started

To get started with the project, follow these steps:

1. **Clone the repository**:
   ```
   git clone <repository-url>
   cd rust-to-ts-swc
   ```

2. **Install Rust**: Ensure you have Rust installed on your machine. You can install it from [rust-lang.org](https://www.rust-lang.org/).

3. **Build the project**:
   ```
   cargo build
   ```

4. **Run the application**:
   ```
   cargo run
   ```

## Usage

The application takes Rust code as input and outputs the corresponding TypeScript code. The generated TypeScript will include comments that reference the original Rust lines for easier tracking and understanding.

## Contributing

Contributions are welcome! Please feel free to submit a pull request or open an issue for any enhancements or bug fixes.

## License

This project is licensed under the MIT License. See the LICENSE file for more details.