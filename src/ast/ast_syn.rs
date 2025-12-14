// Shared re-exports of `syn` AST types so converters can share a single import surface.
pub use syn::{
    File,
    Item,
    ItemFn,
    FnArg,
    ReturnType,
    Type,
    ItemStruct,
    Fields,
    Stmt,
    Expr,
    Block,
    ImplItem,
    Pat,
    PatTuple,
};
