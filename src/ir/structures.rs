use std::fmt;
use std::error::Error;
use std::collections::HashMap;

use slotmap::{basic, new_key_type, SlotMap};
use itertools::Itertools;

use super::values;
use super::types::{Type, TypeKind};


new_key_type! {
    pub struct ValueRef;
    pub struct BlockRef;
    pub struct FunctionRef;
}

#[derive(Debug, Clone)]
pub enum ValueKind {
    ConstantInt(values::ConstantInt),
    ConstantBool(values::ConstantBool),
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

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ValueKind::ConstantInt(inner) => write!(f, "{}", inner.value),
            ValueKind::ConstantBool(inner) => write!(f, "{}", inner.value),
            ValueKind::ConstantNullPtr(_) => write!(f, "null: ptr"),
            ValueKind::ConstantUnit(_) => write!(f, "()"),
            ValueKind::Binary(..) | ValueKind::Offset(..) | ValueKind::FnCall(..) |
            ValueKind::Alloca(..) | ValueKind::Load(..) | ValueKind::Store(..) =>
                write!(f, "%{}: {}", self.name.clone().unwrap_or(String::from("<anonymous>")), self.ty),
            ValueKind::Argument(..) =>
                write!(f, "#{}: {}", self.name.clone().unwrap_or(String::from("<anonymous>")), self.ty)
        }
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

impl Function {
    pub fn get_basic_block(&self, bb_ref: BlockRef) -> &BasicBlock {
        self.blocks_ctx.get(bb_ref).unwrap()
    }

    pub fn get_basic_block_mut(&mut self, bb_ref: BlockRef) -> &mut BasicBlock {
        self.blocks_ctx.get_mut(bb_ref).unwrap()
    }

    pub fn insert_dangling_basic_block(&mut self, bb: BasicBlock) -> BlockRef {
        let handler = self.blocks_ctx.insert(bb);
        handler
    }

    pub fn append_back_dangling_basic_block(&mut self, bb: BlockRef) -> BlockRef {
        assert!(
            self.blocks_ctx.contains_key(bb),
            "try to append a dangling basic block to the wrong function `{}`",
            self.name
        );
        self.blocks.push(bb);
        bb
    }

    /// insert a basic block into function.
    pub fn append_basic_block(&mut self, bb: BasicBlock) -> BlockRef {
        let handler = self.insert_dangling_basic_block(bb);
        self.blocks.push(handler);
        handler
    }
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

    pub fn insert_value(&mut self, value: Value) -> ValueRef {
        self.value_ctx
            .insert(value)
    }

    pub fn append_function(&mut self, function: Function) -> FunctionRef {
        let function_name = function.name.clone();
        let handler = self.func_ctx.insert(function);
        assert!(
            self.string_func_map.insert(function_name.clone(), handler).is_none(),
            "try to insert duplicated function `{}`",
            function_name.clone()
        );
        self.funcs.push(handler);
        handler
    }

    pub fn get_value(&self, value_ref: ValueRef) -> &Value {
        self.value_ctx.get(value_ref).unwrap()
    }

    pub fn get_value_mut(&mut self, value_ref: ValueRef) -> &mut Value {
        self.value_ctx.get_mut(value_ref).unwrap()
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

    pub fn get_function_mut(&mut self, func_ref: FunctionRef) -> &mut Function {
        self.func_ctx.get_mut(func_ref).unwrap()
    }

    pub fn get_function(&self, func_ref: FunctionRef) -> &Function {
        self.func_ctx.get(func_ref).unwrap()
    }
}



impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for funcref in self.funcs.iter() {
            let function = self
                .func_ctx
                .get(funcref.clone())
                .unwrap();
            let param_val = function.args
                .iter()
                .map(| arg_ref | self.get_value(arg_ref.clone()) )
                .collect::<Vec<_>>();
            write!(f, "fn %{}({}) -> {} ",
                    function.name,
                    param_val.iter().format(", "),
                    function.ty.get_function_ret_type().unwrap()
            )?;
            if function.is_external {
                write!(f, ";\n\n")?;
            } else {
                write!(f, "{{\n")?;
                for bb_ref in function.blocks.iter() {
                    let basic_block = function
                        .blocks_ctx
                        .get(bb_ref.clone())
                        .unwrap();
                    write!(f, "%{}:\n", basic_block.name
                        .clone()
                        .unwrap_or(String::from("%<unknown_label>")))?;
                    for value_ref in basic_block.instrs.iter() {
                        let value = self.get_value(value_ref.clone());
                        match &value.kind {
                            ValueKind::Alloca(inner) => {
                                write!(f, "  let {} = alloca {}, {}\n",
                                    value, inner.elem_type, inner.num_elements)
                            },
                            ValueKind::Binary(inner) => {
                                let lhs = self.get_value(inner.lhs);
                                let rhs = self.get_value(inner.rhs);
                                write!(f, "  let {} = {} {}, {}\n",
                                        value, inner.op, lhs, rhs)
                            }
                            _ => panic!("invalid instruction {}", value)
                        }?;
                    };

                    match &basic_block.terminator {
                        Terminator::Panic => write!(f, "  panic!\n"),
                        Terminator::Branch(inner) => {
                            let cond = self.get_value(inner.cond);
                            let true_bb = function.get_basic_block(inner.true_label);
                            let false_bb = function.get_basic_block(inner.false_label);
                            write!(f, "  br {}, label %{}, label %{}\n",
                                    cond,
                                    true_bb.name.clone().unwrap_or(String::from("%<unknown_label>")),
                                    false_bb.name.clone().unwrap_or(String::from("%<unknown_label>"))
                            )
                        },
                        Terminator::Jump(inner) => {
                            let bb = function.get_basic_block(inner.dest);
                            write!(f, "  jmp label %{}\n",
                                    bb.name.clone().unwrap_or(String::from("%<unknown_label>"))
                            )
                        },
                        Terminator::Return(inner) => {
                            let ret = self.get_value(inner.value);
                            write!(f, "  ret {}\n",
                                    ret
                            )
                        }
                    }?;
                }
                write!(f, "}}\n\n")?
            }
        }
        Ok(())
    }
}