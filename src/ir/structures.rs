
use std::collections::HashMap;

use slotmap::{SlotMap, new_key_type};

use super::values;
use super::types::Type;


new_key_type! {
    pub struct ValueRef;
    pub struct BlockRef;
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

pub struct Function {
    pub ty: Type,
    pub name: String,
    pub args: Vec<ValueRef>,

    pub blocks: Vec<BlockRef>,
    pub blocks_ctx: SlotMap<BlockRef, BasicBlock>,
}


pub struct Module {
    pub name: Option<String>,

    pub value_ctx: SlotMap<ValueRef, Value>,
    pub func_ctx: HashMap<String, Function>,

    pub globals: Vec<ValueRef>
}

impl Module {
    pub fn new() -> Module {
        Module {
            name: None,
            value_ctx: SlotMap::with_key(),
            func_ctx: HashMap::new(),
            globals: Vec::new()
        }
    }
}
