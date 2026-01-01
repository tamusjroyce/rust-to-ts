// Converted from Rust: fn compare_and_print(...)
export function compare_and_print(rust_mod: Module, ts_mod: Module): void {
  // Rust macro
  console.log(`=== ast_v2 Rust module ===
${rust_mod}`);
  // Rust macro
  console.log(`=== ast_v2 TS module ===
${ts_mod}`);
  // Rust macro
  console.log(`=== ast_v2 HelloWorld mapping ===`);
  // Unsupported for-loop iterator: & rust_mod . types
  // Original: for rtype in & rust_mod . types { if let Some (ttype) = ts_mod . types . iter () . find (| t | t . name == rtype . name) { println ! ("Type {}: Rust {:?} ↔ TS {:?}" , rtype . name , rtype . kind , ttype . kind) ; for rf in & rtype . fields { match ttype . fields . iter () . find (| tf | tf . name == rf . name) { Some (tf) => { let status = if rf . ty == tf . ty { "OK" } else { "MISMATCH" } ; println ! ("  field {}: Rust {:?} ↔ TS {:?} => {}" , rf . name , rf . ty , tf . ty , status) ; } None => println ! ("  field {}: present in Rust, missing in TS" , rf . name) , } } } else { println ! ("Type {}: present in Rust, missing in TS" , rtype . name) ; } }
  // Unsupported for-loop iterator: & rust_mod . functions
  // Original: for rfn in & rust_mod . functions { if let Some (tfn) = ts_mod . functions . iter () . find (| f | f . name == rfn . name) { println ! ("Function {}:" , rfn . name) ; let param_status = if rfn . params . len () == tfn . params . len () && rfn . params . iter () . zip (& tfn . params) . all (| (rp , tp) | rp . name == tp . name && rp . ty == tp . ty) { "params OK" } else { "params MISMATCH" } ; let ret_status = if rfn . return_type == tfn . return_type { "return type OK" } else { "return type MISMATCH" } ; println ! ("  {} / {}" , param_status , ret_status) ; } else { println ! ("Function {}: present in Rust, missing in TS" , rfn . name) ; } }
}

