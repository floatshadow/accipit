
use std::collections::HashMap;

use slotmap::{SlotMap, new_key_type};

use super::values;
use super::types::Type;


new_key_type! {
    pub struct ValueRef;
    pub struct BlockRef;
    pub struct FunctionRef;
}

#[derive(Debug, Clone)]
pub enum ValueKind {
    ConstantInt(values::ConstantInt),
    ConstantNullPtr(values::ConstantNullPtr),
    ConstantUnit(values::ConstantUnit),
    Argument(values::Argument),
    Binary(values::Binary),
    Offset(values::Offset),
    FnCall(values::FunctionCall),
    Alloca(values::Alloca),
    Load(values::Load),
    Store(values::Store),
}

#[derive(Debug, Clone)]
pub enum Terminator {
    Branch(values::Branch),
    Jump(values::Jump),
    Return(values::Return),
    /* dummy terminator, program will crash if it runs into panic */
    Panic
}

/* In Accipit, the class used to mean a variable (symbol) and the statement that assigns to it is the `Value`.
 */
#[derive(Debug, Clone)]
pub struct Value {
    pub ty: Type,
    pub name: Option<String>,
    pub kind: ValueKind
}

impl Value {
    pub fn new(ty: Type, name: Option<String>, kind: ValueKind) -> Value {
        Value { ty, name, kind }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }

    pub fn isa_instruction(&self) -> bool {
        matches!(self.kind, 
                ValueKind::Binary(..) | ValueKind::Offset(..) | ValueKind::FnCall(..) | 
                ValueKind::Alloca(..) | ValueKind::Load(..) | ValueKind::Store(..))
    }
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub name: Option<String>,

    pub instrs: Vec<ValueRef>,
    pub terminator: Terminator,
}

impl BasicBlock {
    pub fn new() -> BasicBlock {
        BasicBlock {
            name: None,
            instrs: Vec::new(),
            terminator: Terminator::Panic
        }
    }

    pub fn insert_instr_before_terminator(&mut self, instr: ValueRef) {
        self.instrs.push(instr);
    }

    pub fn set_terminator(&mut self, term: Terminator) {
        self.terminator = term;
    }

    pub fn set_name(&mut self, name: Option<String>) {
        self.name = name;
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub ty: Type,
    pub name: String,
    pub args: Vec<ValueRef>,
    pub is_external: bool,

    pub blocks: Vec<BlockRef>,
    pub blocks_ctx: SlotMap<BlockRef, BasicBlock>,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub name: Option<String>,

    pub value_ctx: SlotMap<ValueRef, Value>,
    pub funcs: Vec<FunctionRef>,
    pub func_ctx: SlotMap<FunctionRef, Function>,
    pub string_func_map: HashMap<String, FunctionRef>,

    pub globals: Vec<ValueRef>
}

impl Module {
    pub fn new() -> Module {
        Module {
            name: None,
            value_ctx: SlotMap::with_key(),
            funcs: Vec::new(),
            func_ctx: SlotMap::with_key(),
            string_func_map: HashMap::new(),
            globals: Vec::new()
        }
    }

    pub fn get_value(&self, value_ref: ValueRef) -> Value {
        self.value_ctx.get(value_ref).unwrap().clone()
    }

    pub fn get_value_type(&self, value_ref: ValueRef) -> Type {
        self.value_ctx.get(value_ref).unwrap().ty.clone()
    }

    pub fn get_function_ref(&self, name: &str) -> FunctionRef {
        self.string_func_map
            .get(name)
            .unwrap()
            .clone()
    }
}
