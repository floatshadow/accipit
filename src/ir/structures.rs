
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

pub enum Terminator {
    Branch(values::Branch),
    Jump(values::Jump),
    Return(values::Return)
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

pub struct BasicBlock {
    pub name: Option<String>,

    pub instrs: Vec<ValueRef>,
    pub terminator: Terminator,
}

pub struct Function {
    pub ty: Type,
    pub name: String,
    pub args: Vec<ValueRef>,

    pub blocks: Vec<BasicBlock>,
    pub blocks_ctx: SlotMap<BlockRef, BasicBlock>,

    pub entry_block: BlockRef,
}


pub struct Module {
    name: Option<String>,

    value_ctx: SlotMap<ValueRef, Value>,
    func_ctx: HashMap<String, Function>
}
