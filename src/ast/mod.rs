mod ast_syn;
mod ir;

// Re-export syn AST types and IR so callers can `use crate::ast::*`.
pub use ir::{IrType, rust_type_to_ir};
pub use ast_syn::{File, Item, ItemFn, FnArg, ReturnType, Type, ItemStruct, Fields, Stmt, Expr, Block, ImplItem, Pat, PatTuple};
