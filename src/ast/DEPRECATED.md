# Deprecated AST Converter (Classic)

This `src/ast` module belongs to the original, rule-based Rust→TypeScript converter (`rust-to-ts` binary).

It is **deprecated** in favor of the new, AST v2 pipeline under `src/ast_v2`, which:
- Uses a shared, language-agnostic AST (`Module`, `TypeDecl`, `Function`, etc.).
- Drives the `ast-v2` binary and the example conversions under `conversion/`.

The classic converter is kept only for legacy purposes and existing examples. New work should:
- Prefer `ast_v2` modules and the `ast-v2` binary.
- Avoid adding new features to `src/ast` or `src/converter`.
