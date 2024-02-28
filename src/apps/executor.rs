use core::fmt;
use std::collections::HashMap;

use crate::ir::{structures::*, values};

use slotmap::{SlotMap, SecondaryMap};

#[derive(Debug)]
pub enum ExecutionError {
    SymbolNotFound(String),
    TypeMismatch(Value, Val),
    OffsetInvalidIndex(Value, usize, usize),
    InvalidPointer,
    StuckInPanic,
    NotImplemented(String),
    UnexpectedIncompatibleVal(Val),
    UseUndefinedValue,
}

/// Trace the source of pointer values,
/// including function parameters, local allocas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryObject {
    /// Function scope of the object.
    pub function: String,
    /// Base address value of the object.
    pub base: ValueRef,
    /// Pointer offset within the object.
    pub offset_within: usize,
    /// Size of the memory object
    pub size: usize
}

impl MemoryObject {

    pub fn in_same_object(&self, other: &MemoryObject) -> bool {
        self.base == other.base
    }

    pub fn is_valid_memory(&self) -> bool {
        self.offset_within < self.size
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Val {
    Unit,
    Integer(i64),
    Bool(bool),
    Pointer(MemoryObject),
    /// Function reference
    Function(FunctionRef),
    Undefined,
}


impl Val {
    pub fn compute_binary(
        module: &Module,
        op: values::BinaryOp,
        lhs: &Val,
        rhs: &Val
    ) -> Result<Val, ExecutionError> {
        match op {
            values::BinaryOp::Add => {
                match (lhs, rhs) {
                    (Val::Integer(val1), Val::Integer(val2)) => Ok(Val::Integer(val1 + val2)),
                    _ => Err(ExecutionError::UnexpectedIncompatibleVal(lhs.clone()))
                }
            },
            values::BinaryOp::Sub => {
                match (lhs, rhs) {
                    (Val::Integer(val1), Val::Integer(val2)) => Ok(Val::Integer(val1 - val2)),
                    _ => Err(ExecutionError::UnexpectedIncompatibleVal(lhs.clone()))
                }
            },     
            values::BinaryOp::Mul => {
                match (lhs, rhs) {
                    (Val::Integer(val1), Val::Integer(val2)) => Ok(Val::Integer(val1 * val2)),
                    _ => Err(ExecutionError::UnexpectedIncompatibleVal(lhs.clone()))
                }
            },
            values::BinaryOp::Div => {
                match (lhs, rhs) {
                    (Val::Integer(val1), Val::Integer(val2)) => Ok(Val::Integer(val1 / val2)),
                    _ => Err(ExecutionError::UnexpectedIncompatibleVal(lhs.clone()))
                }
            },
            values::BinaryOp::Rem => {
                match (lhs, rhs) {
                    (Val::Integer(val1), Val::Integer(val2)) => Ok(Val::Integer(val1 % val2)),
                    _ => Err(ExecutionError::UnexpectedIncompatibleVal(lhs.clone()))
                }
            },
            values::BinaryOp::And => {
                match (lhs, rhs) {
                    (Val::Integer(val1), Val::Integer(val2)) => Ok(Val::Integer(val1 & val2)),
                    (Val::Bool(val1), Val::Bool(val2)) => Ok(Val::Bool(val1 & val2)),
                    _ => Err(ExecutionError::UnexpectedIncompatibleVal(lhs.clone()))
                }
            },
            values::BinaryOp::Or => {
                match (lhs, rhs) {
                    (Val::Integer(val1), Val::Integer(val2)) => Ok(Val::Integer(val1 | val2)),
                    (Val::Bool(val1), Val::Bool(val2)) => Ok(Val::Bool(val1 | val2)),
                    _ => Err(ExecutionError::UnexpectedIncompatibleVal(lhs.clone()))
                }
            },
            values::BinaryOp::Xor => {
                match (lhs, rhs) {
                    (Val::Integer(val1), Val::Integer(val2)) => Ok(Val::Integer(val1 ^ val2)),
                    (Val::Bool(val1), Val::Bool(val2)) => Ok(Val::Bool(val1 ^ val2)),
                    _ => Err(ExecutionError::UnexpectedIncompatibleVal(lhs.clone()))
                }
            },
            values::BinaryOp::Lt => {
                match (lhs, rhs) {
                    (Val::Integer(val1), Val::Integer(val2)) => Ok(Val::Bool(val1 < val2)),
                    _ => Err(ExecutionError::UnexpectedIncompatibleVal(lhs.clone()))
                }
            },
            values::BinaryOp::Gt => {
                match (lhs, rhs) {
                    (Val::Integer(val1), Val::Integer(val2)) => Ok(Val::Bool(val1 > val2)),
                    _ => Err(ExecutionError::UnexpectedIncompatibleVal(lhs.clone()))
                }
            },
            values::BinaryOp::Le => {
                match (lhs, rhs) {
                    (Val::Integer(val1), Val::Integer(val2)) => Ok(Val::Bool(val1 <= val2)),
                    _ => Err(ExecutionError::UnexpectedIncompatibleVal(lhs.clone()))
                }
            },
            values::BinaryOp::Ge => {
                match (lhs, rhs) {
                    (Val::Integer(val1), Val::Integer(val2)) => Ok(Val::Bool(val1 >= val2)),
                    _ => Err(ExecutionError::UnexpectedIncompatibleVal(lhs.clone()))
                }
            },
            values::BinaryOp::Eq => {
                match (lhs, rhs) {
                    (Val::Integer(val1), Val::Integer(val2)) => Ok(Val::Bool(val1 == val2)),
                    _ => Err(ExecutionError::UnexpectedIncompatibleVal(lhs.clone()))
                }
            },
            values::BinaryOp::Ne => {
                match (lhs, rhs) {
                    (Val::Integer(val1), Val::Integer(val2)) => Ok(Val::Bool(val1 != val2)),
                    _ => Err(ExecutionError::UnexpectedIncompatibleVal(lhs.clone()))
                }
            }
        }
    }
}

impl Val {
    pub fn matches_value(self, value: &Value) -> Result<Val, ExecutionError> {
        if match &self {
            Val::Integer(..) => value.ty.is_i64_type(),
            Val::Bool(..) => value.ty.is_i1_type(),
            Val::Pointer(..) => value.ty.is_pointer_type(),
            Val::Function(..) => value.ty.is_function_type(),
            Val::Unit => value.ty.is_unit_type(),
            Val::Undefined => false,
        } {
            Ok(self)
        } else {
            Err(ExecutionError::TypeMismatch(value.clone(), self))
        }
    }
}

pub struct ProgramEnv {
    pub val_env: SecondaryMap<ValueRef, Val>,
    pub memory: SecondaryMap<ValueRef, Val>,
    /// current working basic block.
    pub position: Option<BlockRef>,
    /// program counter.
    pub program_counter: Option<ValueRef>,
    pub frames: Vec<FunctionRef>
}

impl ProgramEnv {
    pub fn set_val(&mut self, val_ref: ValueRef, value: Val) -> Option<Val> {
        self.val_env.insert(val_ref, value) 
    }

    pub fn get_val(&self, value_ref: ValueRef) -> &Val {
        self.val_env.get(value_ref).unwrap()
    }
}

impl ProgramEnv {
    pub fn new() -> ProgramEnv {
        ProgramEnv {
            val_env: SecondaryMap::new(),
            memory: SecondaryMap::new(),
            position: None,
            program_counter: None,
            frames: Vec::new()
        }
    }
}


pub fn single_step(
    env: &mut ProgramEnv,
    module: &Module,
    value_data: &Value 
) -> Result<Val, ExecutionError> {
    match &value_data.kind {
        ValueKind::Binary(inner) => {
            let lhs = env.get_val(inner.lhs);
            let rhs = env.get_val(inner.rhs);
            Val::compute_binary(module, inner.op.clone(), lhs, rhs)
        },
        _ => Err(ExecutionError::NotImplemented(String::from("Unhandled Instruction")))
    }
}

pub fn single_step_terminator(
    env: &mut ProgramEnv,
    module: &Module,
    term: &Terminator
) -> Result<Val, ExecutionError> {
    match &term {
        Terminator::Branch(inner) => {
            let cond = env.get_val(inner.cond);
            match cond {
                Val::Bool(true) => {
                    env.position = Some(inner.true_label);
                    Ok(Val::Unit)
                },
                Val::Bool(false) => {
                    env.position = Some(inner.false_label);
                    Ok(Val::Unit)
                },
                _ => Err(ExecutionError::UnexpectedIncompatibleVal(cond.clone()))
            }
        },
        Terminator::Jump(inner) => {
            env.position = Some(inner.dest);
            Ok(Val::Unit)
        },
        Terminator::Return(inner) => {
            env.position = None;
            let ret_val = env.get_val(inner.value);
            Ok(ret_val.clone())
        },
        Terminator::Panic => {
            Err(ExecutionError::StuckInPanic)
        }
    }
}

pub fn run_on_basicblock(
    env: &mut ProgramEnv,
    module: &Module,
    block: &BasicBlock
) -> Result<Val, ExecutionError> {
    for instr in block.instrs.iter().cloned() {
        env.program_counter = Some(instr);
        let value_data = module.get_value(instr);
        let val = single_step(env, module, value_data)?;
        env.set_val(instr, val);
    };
    single_step_terminator(env, module, &block.terminator)
}

pub fn run_on_function(
    env: &mut ProgramEnv,
    module: &Module,
    function: FunctionRef,
    args: Vec<Val>
) -> Result<Val, ExecutionError> {
    env.frames.push(function);
    let function = module.func_ctx.get(function).unwrap();
    // set args values
    let params: Vec<Val> = args
        .into_iter().zip(function.args.iter().cloned())
        .map( | (arg, param) | {
            let value = module.get_value(param);
            arg.matches_value(value)
        } )
        .collect::<Result<_, _>>()?;

    params
        .into_iter().zip(function.args.iter().cloned())
        .for_each(| (val, value) | { env.set_val(value, val); } );

    let entry_bb = function.blocks[0];
    env.position = Some(entry_bb);
    let mut bb_exit_val = Val::Undefined;

    while let Some(current_bb) = env.position {
        let block = function.blocks_ctx.get(current_bb).unwrap();
        bb_exit_val = run_on_basicblock(env, module, block)?;
    };
    Ok(bb_exit_val)

}


pub fn run_on_module(
    env: &mut ProgramEnv,
    module: &Module,
    entry_fn: &str,
    args: Vec<Val>
) -> Result<Val, ExecutionError> {
    let function = module.get_function_ref(entry_fn);
    run_on_function(env, module, function, args)
}