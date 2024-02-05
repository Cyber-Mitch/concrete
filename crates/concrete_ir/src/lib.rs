use std::collections::{BTreeMap, HashMap};

use concrete_ast::common::{Ident, Span};

pub mod lowering;

pub type LocalIndex = usize;
pub type BlockIndex = usize;
pub type TypeIndex = usize;
pub type FieldIndex = usize;

#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    pub symbols: HashMap<DefId, String>,
    pub modules: HashMap<String, DefId>,
    pub functions: HashMap<String, DefId>,
    pub constants: HashMap<String, DefId>,
    pub structs: HashMap<String, DefId>,
    pub types: HashMap<String, DefId>,
}

#[derive(Debug, Clone)]
pub struct ProgramBody {
    pub module_names: BTreeMap<String, DefId>,
    pub modules: BTreeMap<DefId, ModuleBody>,
}

#[derive(Debug, Clone)]
pub struct ModuleBody {
    pub id: DefId,
    pub parent_id: Option<DefId>,
    pub symbols: SymbolTable,
    pub functions: BTreeMap<DefId, FnBody>,
    pub modules: BTreeMap<DefId, ModuleBody>,
}

/// Function body
#[derive(Debug, Clone)]
pub struct FnBody {
    pub id: DefId,
    pub basic_blocks: Vec<BasicBlock>,
    pub locals: Vec<Local>,
}

#[derive(Debug, Clone, Default)]
pub struct BasicBlock {
    pub statements: Vec<Statement>,
    pub terminator: Option<Box<Terminator>>, // should be some once mir is built
}

#[derive(Debug, Clone)]
pub struct Statement {
    pub span: Span,
    pub kind: StatementKind,
}

#[derive(Debug, Clone)]
pub enum StatementKind {
    Assign(Place, Rvalue),
    StorageLive(LocalIndex),
    StorageDead(LocalIndex),
}

#[derive(Debug, Clone)]
pub struct Terminator {
    pub span: Span,
    pub kind: TerminatorKind,
}

#[derive(Debug, Clone)]
pub enum TerminatorKind {
    Goto {
        target: BasicBlock,
    },
    Return,
    Unreachable,
    Call {
        func: DefId,
        args: Vec<Rvalue>,
        destination: Option<Place>, // where return value is stored
        target: Option<BlockIndex>, // where to jump after call, if none diverges
    },
    SwitchInt {
        discriminator: Operand,
        targets: SwitchTargets,
    },
}

#[derive(Debug, Clone)]
pub struct SwitchTargets {
    pub values: Vec<u128>,
    pub targets: Vec<BlockIndex>, // last target is the otherwise block
}

#[derive(Debug, Clone)]
pub enum Rvalue {
    Use(Operand),
    BinaryOp(BinOp, Box<(Rvalue, Rvalue)>),
    UnaryOp(UnOp, Box<Rvalue>),
    Ref(Mutability, Place),
}

#[derive(Debug, Clone)]
pub enum Operand {
    Place(Place),
    Const(ConstData),
}

/// A place in memory
#[derive(Debug, Clone)]
pub struct Place {
    pub local: LocalIndex,
    pub projection: Vec<PlaceElem>,
}

#[derive(Debug, Clone)]
pub enum PlaceElem {
    /// Dereference
    Deref,
    /// Get a field
    Field(FieldIndex, TypeIndex),
    /// array index
    Index(LocalIndex),
}

#[derive(Debug, Clone)]
pub struct Local {
    pub span: Option<Span>,
    pub ty: Ty,
    pub kind: LocalKind,
}

impl Local {
    pub fn new(span: Option<Span>, kind: LocalKind, ty: Ty) -> Self {
        Self { span, kind, ty }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LocalKind {
    /// User-declared variable binding or compiler-introduced temporary.
    Temp,
    /// Function argument.
    Arg,
    /// Location of function's return value.
    ReturnPointer,
}

/// Aggregate data type: struct, enum, tuple..
pub struct AdtBody {
    pub name: DefId,
    pub variants: Vec<VariantDef>,
}

// Definition of a variant, a struct field or enum variant.
pub struct VariantDef {
    pub id: DefId,
    // The relative position in the aggregate structure.
    pub discriminant: usize,
}

// A definition id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct DefId {
    // The program id, like a crate in rust.
    pub program_id: usize,
    pub id: usize,
}

#[derive(Debug, Clone)]
pub struct Ty {
    pub span: Span,
    pub kind: TyKind,
}

impl Ty {
    pub fn new(span: &Span, kind: TyKind) -> Self {
        Self { span: *span, kind }
    }
}

#[derive(Debug, Clone)]
pub enum TyKind {
    Bool,
    Char,
    Int(IntTy),
    Uint(UintTy),
    Float(FloatTy),
    String,
    Array(Box<Ty>, Box<ConstData>),
    Ref(Box<Self>, Mutability),
    // Type param <T>
    Param {
        index: usize,
        name: String, // todo: change me?
    },
}

impl TyKind {
    pub fn get_falsy_value(&self) -> ValueTree {
        match self {
            TyKind::Bool => ValueTree::Leaf(ConstValue::Bool(false)),
            TyKind::Char => todo!(),
            TyKind::Int(ty) => match ty {
                IntTy::I8 => ValueTree::Leaf(ConstValue::I8(0)),
                IntTy::I16 => ValueTree::Leaf(ConstValue::I16(0)),
                IntTy::I32 => ValueTree::Leaf(ConstValue::I32(0)),
                IntTy::I64 => ValueTree::Leaf(ConstValue::I64(0)),
                IntTy::I128 => ValueTree::Leaf(ConstValue::I128(0)),
            },
            TyKind::Uint(ty) => match ty {
                UintTy::U8 => ValueTree::Leaf(ConstValue::U8(0)),
                UintTy::U16 => ValueTree::Leaf(ConstValue::U16(0)),
                UintTy::U32 => ValueTree::Leaf(ConstValue::U32(0)),
                UintTy::U64 => ValueTree::Leaf(ConstValue::U64(0)),
                UintTy::U128 => ValueTree::Leaf(ConstValue::U128(0)),
            },
            TyKind::Float(_) => todo!(),
            TyKind::String => todo!(),
            TyKind::Array(_, _) => todo!(),
            TyKind::Ref(_, _) => todo!(),
            TyKind::Param { index, name } => todo!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Mutability {
    Not,
    Mut,
}

#[derive(Debug, Clone, Copy)]
pub enum IntTy {
    I8,
    I16,
    I32,
    I64,
    I128,
}

#[derive(Debug, Clone, Copy)]
pub enum UintTy {
    U8,
    U16,
    U32,
    U64,
    U128,
}

#[derive(Debug, Clone, Copy)]
pub enum FloatTy {
    F32,
    F64,
}

#[derive(Debug, Clone)]
pub struct ConstData {
    pub ty: TyKind,
    pub data: ConstKind,
}

#[derive(Debug, Clone)]
pub enum ConstKind {
    Param(ParamConst),
    Value(ValueTree),
    Expr(Box<ConstExpr>),
}

#[derive(Debug, Clone)]
pub struct ParamConst {
    pub index: usize,
    pub ident: Ident,
}

#[derive(Debug, Clone)]
// https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/consts/valtree/enum.ValTree.html
pub enum ValueTree {
    Leaf(ConstValue),
    Branch(Vec<Self>),
}

/// Constant expression
#[derive(Debug, Clone)]
pub enum ConstExpr {
    Binop(BinOp, ConstData, ConstData),
    UnOp(UnOp, ConstData),
    FunctionCall(ConstData, Vec<ConstData>),
}

#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    BitXor,
    BitAnd,
    BitOr,
    Shl,
    Shr,
    Eq,
    Lt,
    Le,
    Ne,
    Ge,
    Gt,
}

#[derive(Debug, Clone, Copy)]
pub enum UnOp {
    Not,
    Neg,
}

#[derive(Debug, Clone, Copy)]
pub enum ConstValue {
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    F32(f32),
    F64(f64),
}
