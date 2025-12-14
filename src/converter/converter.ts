// Converted from Rust: fn convert_rust_to_ts(...)
export function convert_rust_to_ts(ast: File, _source: string, is_main_file: boolean): string {
  // Rust variable declaration
  let output = String.new();
  // Rust if
  if (is_main_file) {
  // Rust expression
  output.push_str("import { NeuralNetwork, make_rng_from_args, env, std } from "./lib.ts";

");
  }
  // Unsupported for-loop iterator: & ast . items
  // Original: for item in & ast . items { match item { Item :: Fn (func) => { output . push_str (& convert_function (func)) ; } Item :: Struct (struct_item) => { output . push_str (& convert_struct (struct_item)) ; } Item :: Impl (impl_item) => { if impl_item . trait_ . is_none () { output . push_str (& convert_impl_inherent (impl_item)) ; } else { output . push_str ("// Skipped trait implementation (not yet supported)\n\n") ; } } Item :: Use (_use_item) => { } Item :: Mod (_m) => { } _ => { output . push_str (& format ! ("// Unsupported Rust item: {}\n\n" , quote :: quote ! (# item) . to_string ())) ; } } }
  return output;
}

// Converted from Rust: fn convert_function(...)
export function convert_function(func: ItemFn): string {
  // Rust variable declaration
  let output = String.new();
  // Unsupported statement: output . push_str (& format ! ("// Converted from Rust: fn {}(...)\n" , func . sig . ident))
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust expression
  output.push_str("export ");
  // Unsupported statement: output . push_str (& format ! ("function {}{}(" , func . sig . ident , gen_suffix))
  // Rust variable declaration
  // Unsupported initializer omitted
  // Unsupported statement: output . push_str (& params . join (", "))
  // Rust expression
  output.push_str(")");
  // Unsupported statement: match & func . sig . output { ReturnType :: Type (_ , ty) => { output . push_str (& format ! (": {}" , convert_type (ty))) ; } ReturnType :: Default => { output . push_str (": void") ; } }
  // Rust expression
  output.push_str(" {
");
  // Unsupported statement: output . push_str (& convert_block (& func . block))
  // Rust expression
  output.push_str("}

");
  return output;
}

// Converted from Rust: fn convert_block(...)
export function convert_block(block: Block): string {
  // Rust variable declaration
  let output = String.new();
  // Rust variable declaration
  // Unsupported initializer omitted
  // Unsupported for-loop pattern: for (i , stmt) in block . stmts . iter () . enumerate () { if let Some (last) = last_idx { if i == last { if let Stmt :: Expr (expr , None) = stmt { match expr { Expr :: ForLoop (_) | Expr :: If (_) => { } _ => { let expr_str = convert_expr (expr) ; if expr_str . is_empty () || expr_str . starts_with ("/* Unsupported") { output . push_str ("  // Unsupported trailing expression\n") ; } else { output . push_str ("  return ") ; output . push_str (& expr_str) ; output . push_str (";\n") ; } continue ; } } } } } output . push_str (& convert_stmt (stmt)) ; } . pat
  // Original: for (i , stmt) in block . stmts . iter () . enumerate () { if let Some (last) = last_idx { if i == last { if let Stmt :: Expr (expr , None) = stmt { match expr { Expr :: ForLoop (_) | Expr :: If (_) => { } _ => { let expr_str = convert_expr (expr) ; if expr_str . is_empty () || expr_str . starts_with ("/* Unsupported") { output . push_str ("  // Unsupported trailing expression\n") ; } else { output . push_str ("  return ") ; output . push_str (& expr_str) ; output . push_str (";\n") ; } continue ; } } } } } output . push_str (& convert_stmt (stmt)) ; }
  return output;
}

// Converted from Rust: fn convert_stmt(...)
export function convert_stmt(stmt: Stmt): string {
  return (undefined as any) /* Unsupported expression: match stmt { Stmt :: Macro (stmt_mac) => convert_macro_stmt (stmt_mac) , Stmt :: Local (local) => { let mut result = String :: from ("  // Rust variable declaration\n") ; let (pat_ts , is_tuple , is_mut) = match & local . pat { Pat :: Ident (p) => (p . ident . to_string () , false , p . mutability . is_some ()) , Pat :: Type (pt) => { match & * pt . pat { Pat :: Ident (p) => (p . ident . to_string () , false , p . mutability . is_some ()) , Pat :: Tuple (PatTuple { elems , .. }) => { let mut names : Vec < String > = Vec :: new () ; for (i , elem) in elems . iter () . enumerate () { match elem { Pat :: Ident (pi) => names . push (pi . ident . to_string ()) , _ => names . push (format ! ("_tmp{}" , i)) , } } (format ! ("[{}]" , names . join (", ")) , true , false) } _ => ("_tmp" . to_string () , false , true) , } } Pat :: Tuple (PatTuple { elems , .. }) => { let mut names : Vec < String > = Vec :: new () ; for (i , elem) in elems . iter () . enumerate () { match elem { Pat :: Ident (pi) => names . push (pi . ident . to_string ()) , _ => names . push (format ! ("_tmp{}" , i)) , } } (format ! ("[{}]" , names . join (", ")) , true , false) } _ => ("_tmp" . to_string () , false , true) , } ; let keyword = if is_mut { "let" } else { "const" } ; if let Some (init) = & local . init { let expr_str = convert_expr (& init . expr) ; if expr_str . contains ("Unsupported expression") { result . push_str ("  // Unsupported initializer omitted\n") ; } else { if is_tuple { result . push_str (& format ! ("  {} {} = {} as any;\n" , keyword , pat_ts , expr_str)) ; } else { result . push_str (& format ! ("  {} {} = {};\n" , keyword , pat_ts , expr_str)) ; } } } else { result . push_str (& format ! ("  {} {};\n" , keyword , pat_ts)) ; } result } Stmt :: Expr (expr , semi) => { if let Expr :: ForLoop (fl) = expr { return convert_for_loop (fl) ; } if let Expr :: If (ifexpr) = expr { return convert_if_stmt (ifexpr) ; } let expr_str = convert_expr (expr) ; if expr_str . is_empty () || expr_str . contains ("Unsupported expression") { return format ! ("  // Unsupported statement: {}\n" , quote :: quote ! (# expr) . to_string ()) ; } let mut result = String :: from ("  // Rust expression\n") ; result . push_str (& format ! ("  {}" , expr_str)) ; if semi . is_some () { result . push (';') ; } result . push ('\n') ; result } _ => String :: from ("  // Unsupported statement (unhandled variant)\n") , } */;
}

// Converted from Rust: fn convert_expr(...)
export function convert_expr(expr: Expr): string {
  return (undefined as any) /* Unsupported expression: match expr { Expr :: ForLoop (fl) => { format ! ("(undefined as any) /* Unsupported expression (for-loop): {} */" , quote :: quote ! (# fl) . to_string ()) } Expr :: If (ifexpr) => { format ! ("(undefined as any) /* Unsupported expression (if): {} */" , quote :: quote ! (# ifexpr) . to_string ()) } Expr :: Unary (u) => { use syn :: UnOp ; let inner = convert_expr (& u . expr) ; match u . op { UnOp :: Neg (_) => format ! ("-{}" , inner) , UnOp :: Not (_) => format ! ("!{}" , inner) , _ => "(undefined as any) /* Unsupported expression */" . to_string () , } } Expr :: Lit (lit) => { match & lit . lit { syn :: Lit :: Int (i) => i . base10_digits () . to_string () , syn :: Lit :: Float (f) => f . base10_digits () . to_string () , syn :: Lit :: Str (s) => format ! ("\"{}\"" , s . value ()) , _ => quote :: quote ! (# lit) . to_string () , } } Expr :: Tuple (t) => { let parts : Vec < String > = t . elems . iter () . map (convert_expr) . collect () ; format ! ("[{}]" , parts . join (", ")) } Expr :: Struct (es) => { let mut fields : Vec < String > = Vec :: new () ; for field in & es . fields { let name = match & field . member { syn :: Member :: Named (id) => id . to_string () , syn :: Member :: Unnamed (u) => u . index . to_string () , } ; let value = convert_expr (& field . expr) ; fields . push (format ! ("{}: {}" , name , value)) ; } format ! ("{{ {} }}" , fields . join (", ")) } Expr :: Path (path) => { if let Some (ident) = path . path . get_ident () { let name = ident . to_string () ; if name == "self" { "selfObj" . to_string () } else { name } } else { ts_path_to_string (& path . path) } } Expr :: MethodCall (mc) => { let recv = convert_expr (& mc . receiver) ; let method = mc . method . to_string () ; let args : Vec < String > = mc . args . iter () . map (convert_expr) . collect () ; match method . as_str () { "min" => { if let Some (arg0) = args . get (0) { format ! ("Math.min({}, {})" , recv , arg0) } else { format ! ("/* Unsupported min call: {}.min() */" , recv) } } "max" => { if let Some (arg0) = args . get (0) { format ! ("Math.max({}, {})" , recv , arg0) } else { format ! ("/* Unsupported max call: {}.max() */" , recv) } } _ => format ! ("{}.{}({})" , recv , method , args . join (", ")) , } } Expr :: Field (f) => { let base = convert_expr (& f . base) ; let member = match & f . member { syn :: Member :: Named (id) => id . to_string () , syn :: Member :: Unnamed (u) => u . index . to_string () , } ; format ! ("{}.{}" , base , member) } Expr :: Binary (binary) => { let left = convert_expr (& binary . left) ; let right = convert_expr (& binary . right) ; let op = match binary . op { syn :: BinOp :: Add (_) => "+" , syn :: BinOp :: Sub (_) => "-" , syn :: BinOp :: Mul (_) => "*" , syn :: BinOp :: Div (_) => "/" , syn :: BinOp :: Rem (_) => "%" , syn :: BinOp :: And (_) => "&&" , syn :: BinOp :: Or (_) => "||" , syn :: BinOp :: Eq (_) => "===" , syn :: BinOp :: Ne (_) => "!==" , syn :: BinOp :: Lt (_) => "<" , syn :: BinOp :: Le (_) => "<=" , syn :: BinOp :: Gt (_) => ">" , syn :: BinOp :: Ge (_) => ">=" , _ => "+" , } ; format ! ("{} {} {}" , left , op , right) } Expr :: Call (call) => { let func = convert_expr (& call . func) ; let args : Vec < String > = call . args . iter () . map (| arg | convert_expr (arg)) . collect () ; if func == "make_rng_from_args" { return String :: from ("({ next_f32: (low: number, high: number) => { function mulberry32(a:number){return function(){let t=a+=0x6D2B79F5;t=Math.imul(t^t>>>15,t|1);t^=t+Math.imul(t^t>>>7,t|61);return ((t^t>>>14)>>>0)/4294967296;}} const seed=(globalThis as any).__RUST_TO_TS_SEED>>>0||0xDEADBEEF; const rand=mulberry32(seed); return low + rand() * (high - low); } })") ; } if func == "rng_name_from_args" { return String :: from ("(((globalThis as any).__RUST_TO_TS_RNG||'default') as string)") ; } if func == "String.from" && args . len () == 1 { return args [0] . clone () ; } if func == "NeuralNetwork.random_uniform_f32_with" || func == "NeuralNetwork.random_uniform_f64_with" { let mut a = args . clone () ; if ! a . is_empty () { a . pop () ; } return format ! ("NeuralNetwork.random_uniform({})" , a . join (", ")) ; } format ! ("{}({})" , func , args . join (", ")) } Expr :: Macro (mac) => convert_macro_expr (mac) , Expr :: Return (ret) => { if let Some (expr) = & ret . expr { format ! ("return {}" , convert_expr (expr)) } else { "return" . to_string () } } _ => format ! ("(undefined as any) /* Unsupported expression: {} */" , quote :: quote ! (# expr) . to_string ()) , } */;
}

// Converted from Rust: fn convert_struct(...)
export function convert_struct(struct_item: ItemStruct): string {
  // Rust variable declaration
  let output = String.new();
  // Unsupported statement: output . push_str (& format ! ("// Converted from Rust: struct {}\n" , struct_item . ident))
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  // Unsupported initializer omitted
  // Unsupported statement: output . push_str (& format ! ("interface {}{} {{\n" , struct_item . ident , generic_suffix))
  // Rust if-let
  const __tmp = (undefined as any) /* Unsupported expression: & struct_item . fields */;
  if (__tmp !== undefined) {
    const val = __tmp;
  // Unsupported for-loop iterator: & fields . named
  // Original: for field in & fields . named { let field_name = field . ident . as_ref () . unwrap () ; let field_type = convert_type (& field . ty) ; output . push_str (& format ! ("  {}: {};\n" , field_name , field_type)) ; }
  }
  // Rust expression
  output.push_str("}

");
  // Rust if
  if (struct_item.ident === "NeuralNetwork") {
  // Rust expression
  output.push_str("export const NeuralNetwork: any = {};

");
  }
  return output;
}

// Converted from Rust: fn convert_type(...)
export function convert_type(ty: Type): string {
  // Rust variable declaration
  const ir = rust_type_to_ir(ty);
  return convert_ir_type((undefined as any) /* Unsupported expression: & ir */);
}

// Converted from Rust: fn convert_ir_type(...)
export function convert_ir_type(ir: IrType): string {
  return (undefined as any) /* Unsupported expression: match ir { IrType :: Number => "number" . to_string () , IrType :: String => "string" . to_string () , IrType :: Bool => "boolean" . to_string () , IrType :: Any => "any" . to_string () , IrType :: Vec (inner) => format ! ("{}[]" , convert_ir_type (inner)) , IrType :: Option (inner) => format ! ("{} | undefined" , convert_ir_type (inner)) , IrType :: Custom (name) => name . clone () , } */;
}

// Converted from Rust: fn ts_path_to_string(...)
export function ts_path_to_string(path: Path): string {
  // Rust variable declaration
  let parts = Vec.new();
  // Unsupported for-loop iterator: & path . segments
  // Original: for seg in & path . segments { parts . push (seg . ident . to_string ()) ; }
  return parts.join(".");
}

// Converted from Rust: fn convert_impl_inherent(...)
export function convert_impl_inherent(impl_item: ItemImpl): string {
  // Rust variable declaration
  let out = String.new();
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust if
  if (type_name === "NeuralNetwork") {
  // Rust variable declaration
  let has_new_with_value = false;
  // Rust variable declaration
  let has_random_uniform = false;
  // Unsupported for-loop iterator: & impl_item . items
  // Original: for it in & impl_item . items { if let ImplItem :: Fn (meth) = it { if meth . sig . receiver () . is_none () { let name = meth . sig . ident . to_string () ; if name == "new_with_value" { has_new_with_value = true ; } if name == "random_uniform" { has_random_uniform = true ; } } } }
  // Rust if
  if (has_new_with_value) {
  // Unsupported statement: out . push_str (& format ! ("// Converted from Rust: impl {} associated functions\n" , type_name))
  // Rust expression
  out.push_str("Object.assign(NeuralNetwork, {
");
  // Rust expression
  out.push_str("  new_with_value: function(x_layers: number, y_nodes: number, z_weights: number, value: any) {
");
  // Rust expression
  out.push_str("    const size = x_layers * y_nodes * z_weights;
");
  // Rust expression
  out.push_str("    const data = Array(size).fill(value);
");
  // Rust expression
  out.push_str("    return {
");
  // Rust expression
  out.push_str("      x_layers, y_nodes, z_weights, data,
");
  // Rust expression
  out.push_str("      dims() { return [this.x_layers, this.y_nodes, this.z_weights]; },
");
  // Rust expression
  out.push_str("      len() { return this.data.length; },
");
  // Rust expression
  out.push_str("      is_empty() { return this.data.length === 0; },
");
  // Rust expression
  out.push_str("      index(x: number, y: number, z: number) {
");
  // Rust expression
  out.push_str("        if (x < this.x_layers && y < this.y_nodes && z < this.z_weights) {
");
  // Rust expression
  out.push_str("          const yz = this.y_nodes * this.z_weights;
");
  // Rust expression
  out.push_str("          return x * yz + y * this.z_weights + z;
");
  // Rust expression
  out.push_str("        }
");
  // Rust expression
  out.push_str("        return undefined;
");
  // Rust expression
  out.push_str("      },
");
  // Rust expression
  out.push_str("      get(x: number, y: number, z: number) {
");
  // Rust expression
  out.push_str("        const i = this.index(x, y, z);
");
  // Rust expression
  out.push_str("        return i === undefined ? undefined : this.data[i];
");
  // Rust expression
  out.push_str("      },
");
  // Rust expression
  out.push_str("      get_mut(x: number, y: number, z: number) {
");
  // Rust expression
  out.push_str("        const i = this.index(x, y, z);
");
  // Rust expression
  out.push_str("        return i === undefined ? undefined : this.data[i];
");
  // Rust expression
  out.push_str("      },
");
  // Rust expression
  out.push_str("    };
");
  // Rust expression
  out.push_str("  },
");
  // Rust expression
  out.push_str("});

");
  }
  // Rust if
  if (has_random_uniform) {
  // Unsupported statement: out . push_str (& format ! ("// Converted from Rust: impl {} associated functions\n" , type_name))
  // Rust expression
  out.push_str("Object.assign(NeuralNetwork, {
");
  // Rust expression
  out.push_str("  random_uniform: function(x_layers: number, y_nodes: number, z_weights: number, low: number, high: number) {
");
  // Rust expression
  out.push_str("    function mulberry32Factory(a:number){return function(){let t=a+=0x6D2B79F5;t=Math.imul(t^t>>>15,t|1);t^=t+Math.imul(t^t>>>7,t|61);return ((t^t>>>14)>>>0)/4294967296;}}
");
  // Rust expression
  out.push_str("    function splitmix64Factory(seed: bigint){
");
  // Rust expression
  out.push_str("      let state = seed;
");
  // Rust expression
  out.push_str("      const MASK = (1n<<64n)-1n;
");
  // Rust expression
  out.push_str("      return function(){
");
  // Rust expression
  out.push_str("        state = (state + 0x9E3779B97F4A7C15n) & MASK;
");
  // Rust expression
  out.push_str("        let z = state;
");
  // Rust expression
  out.push_str("        z ^= z >> 30n; z = (z * 0xBF58476D1CE4E5B9n) & MASK;
");
  // Rust expression
  out.push_str("        z ^= z >> 27n; z = (z * 0x94D049BB133111EBn) & MASK;
");
  // Rust expression
  out.push_str("        z ^= z >> 31n;
");
  // Rust expression
  out.push_str("        const u = Number(z >> 11n) / 9007199254740992; // 2^53
");
  // Rust expression
  out.push_str("        return u;
");
  // Rust expression
  out.push_str("      }
");
  // Rust expression
  out.push_str("    }
");
  // Rust expression
  out.push_str("    function rotl32(x:number, n:number){ x = x>>>0; return ((x<<n) | (x>>> (32-n)))>>>0; }
");
  // Rust expression
  out.push_str("    function chacha8Factory(seedU64: bigint){
");
  // Rust expression
  out.push_str("      const st = new Uint32Array(16);
");
  // Rust expression
  out.push_str("      st[0]=0x61707865; st[1]=0x3320646e; st[2]=0x79622d32; st[3]=0x6b206574;
");
  // Rust expression
  out.push_str("      // Mulberry64-like (splitmix64-based) generator to derive key words via f32 cast pattern
");
  // Rust expression
  out.push_str("      const sm = (function(){ let s=seedU64; const MASK=(1n<<64n)-1n; return function(){ s=(s+0x9E3779B97F4A7C15n)&MASK; let z=s; z^=z>>30n; z=(z*0xBF58476D1CE4E5B9n)&MASK; z^=z>>27n; z=(z*0x94D049BB133111EBn)&MASK; z^=z>>31n; const u=Number(z>>11n)/9007199254740992; return u; };})();
");
  // Rust expression
  out.push_str("      const F32_MAX = 340282346638528859811704183484516925440.0; // f32::MAX as f64
");
  // Rust expression
  out.push_str("      for (let i=0;i<8;i++){
");
  // Rust expression
  out.push_str("        // Emulate sm.next_f32(0.0, f32::MAX) as u32 with saturating cast
");
  // Rust expression
  out.push_str("        let val = sm()*F32_MAX;
");
  // Rust expression
  out.push_str("        let u32 = 0; if (!Number.isFinite(val) || val<=0){ u32=0; } else if (val>=4294967295){ u32=4294967295; } else { u32 = Math.floor(val)>>>0; }
");
  // Rust expression
  out.push_str("        const tweak = Math.imul(0x9E3779B9>>>0, (i+1)>>>0)>>>0;
");
  // Rust expression
  out.push_str("        st[4+i] = (u32 ^ tweak)>>>0;
");
  // Rust expression
  out.push_str("      }
");
  // Rust expression
  out.push_str("      st[12]=0; st[13]=0;
");
  // Rust expression
  out.push_str("      const seedLo = Number(seedU64 & 0xFFFFFFFFn)>>>0;
");
  // Rust expression
  out.push_str("      const seedHi = Number((seedU64>>32n) & 0xFFFFFFFFn)>>>0;
");
  // Rust expression
  out.push_str("      st[14] = (seedLo ^ 0xDEADBEEF)>>>0;
");
  // Rust expression
  out.push_str("      st[15] = (seedHi ^ 0xBADC0FFE)>>>0;
");
  // Rust expression
  out.push_str("      const buf = new Uint32Array(16); let idx=16;
");
  // Rust expression
  out.push_str("      function qr(x:Uint32Array,a:number,b:number,c:number,d:number){ x[a]=(x[a]+x[b])>>>0; x[d]^=x[a]; x[d]=rotl32(x[d],16); x[c]=(x[c]+x[d])>>>0; x[b]^=x[c]; x[b]=rotl32(x[b],12); x[a]=(x[a]+x[b])>>>0; x[d]^=x[a]; x[d]=rotl32(x[d],8); x[c]=(x[c]+x[d])>>>0; x[b]^=x[c]; x[b]=rotl32(x[b],7); }
");
  // Rust expression
  out.push_str("      function refill(){
");
  // Rust expression
  out.push_str("        const x = new Uint32Array(st);
");
  // Rust expression
  out.push_str("        for (let r=0;r<8;r++){
");
  // Rust expression
  out.push_str("          // column rounds
");
  // Rust expression
  out.push_str("          qr(x,0,4,8,12); qr(x,1,5,9,13); qr(x,2,6,10,14); qr(x,3,7,11,15);
");
  // Rust expression
  out.push_str("          // diagonal rounds
");
  // Rust expression
  out.push_str("          qr(x,0,5,10,15); qr(x,1,6,11,12); qr(x,2,7,8,13); qr(x,3,4,9,14);
");
  // Rust expression
  out.push_str("        }
");
  // Rust expression
  out.push_str("        for (let i=0;i<16;i++){ buf[i] = (x[i] + st[i])>>>0; }
");
  // Rust expression
  out.push_str("        st[12] = (st[12] + 1)>>>0; if (st[12]===0){ st[13] = (st[13]+1)>>>0; }
");
  // Rust expression
  out.push_str("        idx=0;
");
  // Rust expression
  out.push_str("      }
");
  // Rust expression
  out.push_str("      function next_u32(){ if (idx>=16) refill(); const v = buf[idx]; idx++; return v>>>0; }
");
  // Rust expression
  out.push_str("      return function(){ const r = next_u32() / 4294967296; return r; };
");
  // Rust expression
  out.push_str("    }
");
  // Rust expression
  out.push_str("    const g:any = globalThis as any;
");
  // Rust expression
  out.push_str("    const rngName = (g.__RUST_TO_TS_RNG || '').toString().toLowerCase();
");
  // Rust expression
  out.push_str("    const seed32 = (g.__RUST_TO_TS_SEED >>> 0) || 0xDEADBEEF;
");
  // Rust expression
  out.push_str("    const seedU64: bigint = (typeof g.__RUST_TO_TS_SEED_U64 !== 'undefined') ? BigInt(g.__RUST_TO_TS_SEED_U64) : BigInt(seed32 >>> 0);
");
  // Rust expression
  out.push_str("    let rand: () => number;
");
  // Rust expression
  out.push_str("    if (rngName === 'mulberry64') { rand = splitmix64Factory(seedU64); }
");
  // Rust expression
  out.push_str("    else if (rngName === 'pcg64') { rand = splitmix64Factory(seedU64); /* TODO: implement PCG64 */ }
");
  // Rust expression
  out.push_str("    else if (rngName === 'chacha8' || rngName === 'chacha8rng') { rand = chacha8Factory(seedU64); }
");
  // Rust expression
  out.push_str("    else { rand = mulberry32Factory(seed32); }
");
  // Rust expression
  out.push_str("    const size = x_layers * y_nodes * z_weights;
");
  // Rust expression
  out.push_str("    const data: number[] = new Array(size);
");
  // Rust expression
  out.push_str("    for (let i = 0; i < size; i++) { const r = rand(); data[i] = low + r * (high - low); }
");
  // Rust expression
  out.push_str("    return {
");
  // Rust expression
  out.push_str("      x_layers, y_nodes, z_weights, data,
");
  // Rust expression
  out.push_str("      dims() { return [this.x_layers, this.y_nodes, this.z_weights]; },
");
  // Rust expression
  out.push_str("      len() { return this.data.length; },
");
  // Rust expression
  out.push_str("      is_empty() { return this.data.length === 0; },
");
  // Rust expression
  out.push_str("      index(x: number, y: number, z: number) {
");
  // Rust expression
  out.push_str("        if (x < this.x_layers && y < this.y_nodes && z < this.z_weights) {
");
  // Rust expression
  out.push_str("          const yz = this.y_nodes * this.z_weights;
");
  // Rust expression
  out.push_str("          return x * yz + y * this.z_weights + z;
");
  // Rust expression
  out.push_str("        }
");
  // Rust expression
  out.push_str("        return undefined;
");
  // Rust expression
  out.push_str("      },
");
  // Rust expression
  out.push_str("      get(x: number, y: number, z: number) {
");
  // Rust expression
  out.push_str("        const i = this.index(x, y, z);
");
  // Rust expression
  out.push_str("        return i === undefined ? undefined : this.data[i];
");
  // Rust expression
  out.push_str("      },
");
  // Rust expression
  out.push_str("      get_mut(x: number, y: number, z: number) {
");
  // Rust expression
  out.push_str("        const i = this.index(x, y, z);
");
  // Rust expression
  out.push_str("        return i === undefined ? undefined : this.data[i];
");
  // Rust expression
  out.push_str("      },
");
  // Rust expression
  out.push_str("    };
");
  // Rust expression
  out.push_str("  },
");
  // Rust expression
  out.push_str("});

");
  }
  }
  // Unsupported for-loop iterator: & impl_item . items
  // Original: for it in & impl_item . items { if let ImplItem :: Fn (meth) = it { let fn_name = format ! ("{}_{}" , type_name , meth . sig . ident) ; let method_generics : Vec < String > = meth . sig . generics . type_params () . map (| tp | tp . ident . to_string ()) . collect () ; let mut all_generics : Vec < String > = generics . clone () ; for g in method_generics { if ! all_generics . iter () . any (| e | e == & g) { all_generics . push (g) ; } } let fn_generic_suffix = if all_generics . is_empty () { String :: new () } else { format ! ("<{}>" , all_generics . join (", ")) } ; let mut params : Vec < String > = Vec :: new () ; if let Some (_receiver) = meth . sig . receiver () { let self_ty = format ! ("{}{}" , type_name , generic_suffix) ; let self_param = format ! ("selfObj: {}" , self_ty) ; params . push (self_param) ; } for input in meth . sig . inputs . iter () { if let FnArg :: Typed (pat_type) = input { let param_name = match & * pat_type . pat { syn :: Pat :: Ident (ident) => ident . ident . to_string () , _ => quote :: quote ! (# pat_type . pat) . to_string () , } ; let param_type = convert_type (& pat_type . ty) ; params . push (format ! ("{}: {}" , param_name , param_type)) ; } } let ret_type = match & meth . sig . output { ReturnType :: Type (_ , ty) => format ! (": {}" , convert_type (ty)) , ReturnType :: Default => ": void" . to_string () , } ; out . push_str (& format ! ("// Converted from Rust: impl {}{}::{}\n" , type_name , generic_suffix , meth . sig . ident)) ; out . push_str (& format ! ("function {}{}({}){} {{\n" , fn_name , fn_generic_suffix , params . join (", ") , ret_type)) ; out . push_str (& convert_block (& meth . block)) ; out . push_str ("}\n\n") ; } }
  return out;
}

// Converted from Rust: fn convert_macro_stmt(...)
export function convert_macro_stmt(stmt_mac: StmtMacro): string {
  // Rust variable declaration
  // Unsupported initializer omitted
  return (undefined as any) /* Unsupported expression: match name . as_str () { "println" => { let (fmt , args) = parse_format_macro_args (& stmt_mac . mac . tokens) ; let mut out = String :: from ("  // Rust macro\n") ; if let Some (fmt_str) = fmt { out . push_str (& format ! ("  console.log({});\n" , build_ts_template_string (& fmt_str , & args))) ; } else { out . push_str (& format ! ("  console.log({}); // raw args\n  // Original: {}\n" , stmt_mac . mac . tokens . to_string () , quote :: quote ! (# stmt_mac) . to_string ())) ; } out } "print" => { let (fmt , args) = parse_format_macro_args (& stmt_mac . mac . tokens) ; let mut out = String :: from ("  // Rust macro\n") ; if let Some (fmt_str) = fmt { out . push_str (& format ! ("  Deno.stdout.writeSync(new TextEncoder().encode({}));\n" , build_ts_template_string (& fmt_str , & args))) ; } else { out . push_str (& format ! ("  Deno.stdout.writeSync(new TextEncoder().encode(String({})));\n  // Original: {}\n" , stmt_mac . mac . tokens . to_string () , quote :: quote ! (# stmt_mac) . to_string ())) ; } out } _ => { format ! ("  // Unsupported macro: {}!\\n  // Original: {}\n" , name , quote :: quote ! (# stmt_mac) . to_string ()) } } */;
}

// Converted from Rust: fn convert_macro_expr(...)
export function convert_macro_expr(mac: ExprMacro): string {
  // Rust variable declaration
  const macro_name = mac.mac.path.segments.last().unwrap().ident.to_string();
  return (undefined as any) /* Unsupported expression: match macro_name . as_str () { "println" => { let (fmt , args) = parse_format_macro_args (& mac . mac . tokens) ; if let Some (fmt_str) = fmt { build_ts_template_string (& fmt_str , & args) } else { format ! ("console.log({})" , mac . mac . tokens . to_string ()) } } "print" => { let (fmt , args) = parse_format_macro_args (& mac . mac . tokens) ; if let Some (fmt_str) = fmt { format ! ("Deno.stdout.writeSync(new TextEncoder().encode({}))" , build_ts_template_string (& fmt_str , & args)) } else { format ! ("Deno.stdout.writeSync(new TextEncoder().encode(String({})))" , mac . mac . tokens . to_string ()) } } _ => { format ! ("(undefined as any) /* Unsupported macro: {}! - Original: {} */" , macro_name , quote :: quote ! (# mac) . to_string ()) } } */;
}

// Converted from Rust: fn parse_format_macro_args(...)
export function parse_format_macro_args(tokens: TokenStream): any {
  // Rust variable declaration
  const parser = Punctuated.parse_terminated;
  // Rust if-let
  const __tmp = parser.parse2(tokens.clone());
  if (__tmp !== undefined) {
    const val = __tmp;
  // Rust variable declaration
  let it = list.iter();
  // Rust if-let
  const __tmp = it.next();
  if (__tmp !== undefined) {
    const first = __tmp;
  // Rust if-let
  const __tmp = first;
  if (__tmp !== undefined) {
    const val = __tmp;
  // Rust if-let
  const __tmp = (undefined as any) /* Unsupported expression: & expr_lit . lit */;
  if (__tmp !== undefined) {
    const val = __tmp;
  // Rust variable declaration
  const fmt = s.value();
  // Rust variable declaration
  let args = Vec.new();
  // Unsupported for-loop iterator: it
  // Original: for a in it { args . push (convert_expr (a)) ; }
  // Rust expression
  return [Some(fmt), args];
  }
  }
  }
  // Rust variable declaration
  const args = list.iter().map(convert_expr).collect();
  // Rust expression
  return [None, args];
  }
  return [None, Vec.new()];
}

// Converted from Rust: fn build_ts_template_string(...)
export function build_ts_template_string(fmt: string, args: any): string {
  // Rust variable declaration
  const s = fmt.replace("{{", "{{}").replace("}}", "{}}");
  // Rust variable declaration
  let result = "`";
  // Rust variable declaration
  let arg_iter = args.iter();
  // Rust variable declaration
  const chars = s.chars().collect();
  // Rust variable declaration
  let i = 0;
  // Unsupported statement: while i < chars . len () { if chars [i] == '{' { let mut j = i + 1 ; while j < chars . len () && chars [j] != '}' { j += 1 ; } if j < chars . len () && chars [j] == '}' { if let Some (arg) = arg_iter . next () { result . push_str ("${") ; result . push_str (arg) ; result . push ('}') ; } else { result . push ('{') ; result . push ('}') ; } i = j + 1 ; continue ; } } if chars [i] == '`' { result . push_str ("\\`") ; } else if chars [i] == '$' { result . push_str ("\\$") ; } else { result . push (chars [i]) ; } i += 1 ; }
  // Rust expression
  result.push('`');
  return result;
}

// Converted from Rust: fn convert_for_loop(...)
export function convert_for_loop(fl: ExprForLoop): string {
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  let out = String.new();
  // Rust expression
  out.push_str("  // Rust for-loop
");
  // Rust if
  if (inclusive) {
  // Unsupported statement: out . push_str (& format ! ("  for (let {} = {}; {} <= {}; {}++) {{\n" , pat_name , start_expr , pat_name , end_expr , pat_name))
  } else {
  // Unsupported statement: out . push_str (& format ! ("  for (let {} = {}; {} < {}; {}++) {{\n" , pat_name , start_expr , pat_name , end_expr , pat_name))
  }
  // Unsupported statement: out . push_str (& convert_block (& fl . body))
  // Rust expression
  out.push_str("  }
");
  return out;
}

// Converted from Rust: fn convert_if_stmt(...)
export function convert_if_stmt(ifexpr: ExprIf): string {
  // Rust if-let
  const __tmp = (undefined as any) /* Unsupported expression: & * ifexpr . cond */;
  if (__tmp !== undefined) {
    const val = __tmp;
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  let out = String.new();
  // Rust expression
  out.push_str("  // Rust if-let
");
  // Unsupported statement: out . push_str (& format ! ("  const __tmp = {};\n" , value_expr))
  // Rust expression
  out.push_str("  if (__tmp !== undefined) {
");
  // Unsupported statement: out . push_str (& format ! ("    const {} = __tmp;\n" , pat_name))
  // Unsupported statement: out . push_str (& convert_block (& ifexpr . then_branch))
  // Rust if-let
  const __tmp = (undefined as any) /* Unsupported expression: & ifexpr . else_branch */;
  if (__tmp !== undefined) {
    const val = __tmp;
  return (undefined as any) /* Unsupported expression: match & * * else_expr { Expr :: Block (b) => { out . push_str ("  } else {\n") ; out . push_str (& convert_block (& b . block)) ; out . push_str ("  }\n") ; } other => { out . push_str ("  } else {\n") ; out . push_str (& format ! ("    // Unsupported else expression: {}\n" , quote :: quote ! (# other) . to_string ())) ; out . push_str ("  }\n") ; } } */;
  } else {
  // Rust expression
  out.push_str("  }
");
  }
  // Rust expression
  return out;
  }
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  let out = String.new();
  // Rust expression
  out.push_str("  // Rust if
");
  // Unsupported statement: out . push_str (& format ! ("  if ({}) {{\n" , cond))
  // Unsupported statement: out . push_str (& convert_block (& ifexpr . then_branch))
  // Rust if-let
  const __tmp = (undefined as any) /* Unsupported expression: & ifexpr . else_branch */;
  if (__tmp !== undefined) {
    const val = __tmp;
  return (undefined as any) /* Unsupported expression: match & * * else_expr { Expr :: Block (b) => { out . push_str ("  } else {\n") ; out . push_str (& convert_block (& b . block)) ; out . push_str ("  }\n") ; } other => { out . push_str ("  } else {\n") ; out . push_str (& format ! ("    // Unsupported else expression: {}\n" , quote :: quote ! (# other) . to_string ())) ; out . push_str ("  }\n") ; } } */;
  } else {
  // Rust expression
  out.push_str("  }
");
  }
  return out;
}

// Converted from Rust: fn convert_rust_file_to_ts_file(...)
export function convert_rust_file_to_ts_file<P>(rs_path: P): Result {
  // Rust variable declaration
  const rs_path = rs_path.as_ref();
  // Rust if
  if (!rs_path.is_file()) {
  // Rust expression
  return Err((undefined as any) /* Unsupported macro: format! - Original: format ! ("Not a file: {}" , rs_path . display ()) */.into());
  }
  // Rust if
  if (rs_path.extension().and_then((undefined as any) /* Unsupported expression: | s | s . to_str () */) !== Some("rs")) {
  // Rust expression
  return Err((undefined as any) /* Unsupported macro: format! - Original: format ! ("Not a .rs file: {}" , rs_path . display ()) */.into());
  }
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  let ts_path = rs_path.to_path_buf();
  // Rust expression
  ts_path.set_extension("ts");
  // Unsupported statement: fs :: write (& ts_path , ts) ?
  return Ok(ts_path);
}

// Converted from Rust: fn is_ignored_dir(...)
export function is_ignored_dir(name: string): boolean {
  return (undefined as any) /* Unsupported macro: matches! - Original: matches ! (name , "target" | ".git" | "node_modules") */;
}

// Converted from Rust: fn convert_rs_dir_to_ts_side_by_side(...)
export function convert_rs_dir_to_ts_side_by_side<P>(root: P): Result {
  // Rust variable declaration
  const root = root.as_ref();
  // Rust variable declaration
  let written = Vec.new();
  // Rust if
  if (root.is_file()) {
  // Rust if
  if (root.extension().and_then((undefined as any) /* Unsupported expression: | s | s . to_str () */) === Some("rs")) {
  // Unsupported statement: written . push (convert_rust_file_to_ts_file (root) ?)
  }
  // Rust expression
  return Ok(written);
  }
  // Rust if
  if (!root.is_dir()) {
  // Rust expression
  return Err((undefined as any) /* Unsupported macro: format! - Original: format ! ("Path not found or not accessible: {}" , root . display ()) */.into());
  }
  // Rust variable declaration
  let stack = (undefined as any) /* Unsupported macro: vec! - Original: vec ! [root . to_path_buf ()] */;
  // Unsupported statement: while let Some (dir) = stack . pop () { for entry in fs :: read_dir (& dir) ? { let entry = entry ? ; let path = entry . path () ; let name = entry . file_name () ; let name = name . to_string_lossy () ; if path . is_dir () { if is_ignored_dir (& name) { continue ; } stack . push (path) ; } else if path . extension () . and_then (| s | s . to_str ()) == Some ("rs") { let ts_path = convert_rust_file_to_ts_file (& path) ? ; written . push (ts_path) ; } } }
  return Ok(written);
}

