use std::fmt;
use std::collections::HashMap;
use std::str::FromStr;

use crate::ir::{structures::*, values::{self, ConstantInt}};

use slotmap::{SlotMap, SecondaryMap};

#[derive(Debug)]
pub enum ExecutionError {
    SymbolNotFound(String),
    TypeMismatch(Value, Val),
    OffsetInvalidIndex(Value, Val, Option<usize>),
    OffsetExceedMemoryRegion(Value),
    InvalidPointer(Value, Val),
    StuckInPanic,
    NotImplemented(String),
    UnexpectedIncompatibleVal(Val),
    UseUndefinedValue,
    InvalidInputArguments(String),
    LexerError,
    ParseError
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

    pub fn try_from_offset(
        base: &MemoryObject,
        offset: usize
    ) -> Option<MemoryObject> {
        let total_offset = base.offset_within + offset;
        if total_offset < base.size {
            Some(MemoryObject {
                function: base.function.clone(),
                base: base.base,
                offset_within: total_offset,
                size: base.size
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Val {
    Unit,
    Integer(i32),
    Bool(bool),
    Pointer(MemoryObject),
    /// Function reference
    Function(String),
    Undefined,
}

impl FromStr for Val {
    type Err = ExecutionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let int_value = s.parse::<i32>()
            .map_err(| _ | ExecutionError::InvalidInputArguments(String::from(s)))?;

        Ok(Val::Integer(int_value))
    }
}

impl fmt::Display for Val {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Val::Unit => write!(f, "()"),
            Val::Integer(inner) => write!(f, "{}", inner),
            Val::Bool(inner) => write!(f, "{}", inner),
            Val::Pointer(inner) => write!(f, "<inner pointer>: {:?}", inner),
            Val::Function(name) => write!(f, "function: {}", name),
            Val::Undefined => write!(f, "<undefined>")
        }
    }
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
            Val::Integer(..) => value.ty.is_i32_type(),
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

pub struct Frame {
    pub frame_val_env: SecondaryMap<ValueRef, Val>,
    pub frame_memory: SecondaryMap<ValueRef, Vec<Val>>,
    pub working_function: Option<FunctionRef>
}

impl Frame {
    pub fn new(working_function: FunctionRef) -> Frame {
        Frame {
            frame_val_env: SecondaryMap::new(),
            frame_memory: SecondaryMap::new(),
            working_function: Some(working_function)
        }
    }

    pub fn new_global() -> Frame {
        Frame {
            frame_val_env: SecondaryMap::new(),
            frame_memory: SecondaryMap::new(),
            working_function: None
        }
    }

    pub fn set_local_val(&mut self, val_ref: ValueRef, value: Val) -> Option<Val> {
        self.frame_val_env.insert(val_ref, value) 
    }

    pub fn get_local_val(&self, value_ref: ValueRef) -> &Val {
        self.frame_val_env.get(value_ref).unwrap()
    }

    pub fn get_local_memory(&self, base: ValueRef) -> &Vec<Val> {
        self.frame_memory.get(base).unwrap()
    }

    pub fn get_local_memory_mut(&mut self, base: ValueRef) -> &mut Vec<Val> {
        self.frame_memory.get_mut(base).unwrap()
    }
}

pub struct ProgramEnv {
    /// current working basic block.
    pub position: Option<BlockRef>,
    /// program counter.
    pub program_counter: Option<ValueRef>,
    pub global_frame: Frame,
    pub frames: Vec<Frame>
}

impl ProgramEnv {
    pub fn get_top_frame(&self) -> Option<&Frame> {
        self.frames.last()
    }

    pub fn get_top_frame_mut(&mut self) -> Option<&mut Frame> {
        self.frames.last_mut()
    }

    pub fn get_global_frame(&self) -> &Frame {
        &self.global_frame
    }

    pub fn get_global_frame_mut(&mut self) -> &mut Frame {
        &mut self.global_frame
    }

    fn search_value_env(
        &self,
        val_ref: ValueRef
    ) -> Option<&Frame> {
        let top_frame = self.get_top_frame().expect("no active frame");
        if top_frame.frame_val_env.contains_key(val_ref) {
            Some(top_frame)
        } else {
            let global_frame = self.get_global_frame();
            if global_frame.frame_val_env.contains_key(val_ref) {
                Some(global_frame)
            } else {
                None
            }
        }
        
    }

    fn search_value_env_mut(
        &mut self,
        val_ref: ValueRef
    ) -> Option<&mut Frame> {
        let top_frame = self.get_top_frame().expect("no active frame");
        if top_frame.frame_val_env.contains_key(val_ref) {
            self.get_top_frame_mut()
        } else {
            let global_frame = self.get_global_frame();
            if global_frame.frame_val_env.contains_key(val_ref) {
                Some(self.get_global_frame_mut())
            } else {
                // slient return top frame assume set the `val`` of a new `value`
                self.get_top_frame_mut()
            }
        }
    }

    pub fn set_val(&mut self, val_ref: ValueRef, value: Val) -> Option<Val> {
        self.search_value_env_mut(val_ref)
            .expect("cannot find value in current scope")
            .set_local_val(val_ref, value)
    }

    pub fn get_val(&self, val_ref: ValueRef) -> &Val {
        self.search_value_env(val_ref)
            .expect("cannot find value in current scope")
            .get_local_val(val_ref)
    }

    pub fn get_top_function<'a>(&'a self, module: &'a Module) -> &Function {
        module.get_function(
            self.get_top_frame()
            .expect("no active frame")
            .working_function
            .expect("global variable scope, no function is active"))
    }

    pub fn get_memory(&self, base: ValueRef) -> &Vec<Val> {
        self.search_value_env(base)
            .expect("cannot find value in current scope")
            .get_local_memory(base)
    }

    pub fn get_memory_mut(&mut self, base: ValueRef) -> &mut Vec<Val> {
        self.search_value_env_mut(base)
            .expect("cannot find value in current scope")
            .get_local_memory_mut(base)
    }

    pub fn initialize_memory(&mut self, base: ValueRef, size: usize) {
        let unitialized_object = vec![Val::Undefined; size];
        self.search_value_env_mut(base)
            .expect("cannot find value in current scope")
            .frame_memory
            .insert(base, unitialized_object);
    }
}

impl ProgramEnv {
    pub fn new() -> ProgramEnv {
        ProgramEnv {
            position: None,
            program_counter: None,
            global_frame: Frame::new_global(),
            frames: Vec::new()
        }
    }
}


pub fn single_step(
    env: &mut ProgramEnv,
    module: &Module,
    value: ValueRef 
) -> Result<Val, ExecutionError> {
    let value_data = module.get_value(value);
    // println!("single step on `{}`", value_data);
    match &value_data.kind {
        ValueKind::Binary(inner) => {
            let lhs = env.get_val(inner.lhs);
            let rhs = env.get_val(inner.rhs);
            Val::compute_binary(module, inner.op.clone(), lhs, rhs)
        },
        ValueKind::Offset(inner) => {
            let base_addr_value = module.get_value(inner.base_addr);
            let base_addr_val = env
                .get_val(inner.base_addr)
                .clone()
                .matches_value(base_addr_value)?;
            
            // bound checking
            let indices: Vec<usize> = inner.index
                .iter().cloned().zip(inner.bounds.iter().cloned())
                .map(| (index, bound) | {
                    let index_val = env.get_val(index);
                    match &index_val {
                        Val::Integer(index_inner) => {
                            let try_usize_index = usize::try_from(index_inner.clone());
                            match (try_usize_index, bound) {
                                (Ok(converted_index), Some(inner_bound)) if converted_index < inner_bound =>
                                    Ok(converted_index),
                                (Ok(converted_index), None) =>
                                    Ok(converted_index),
                                _ => Err(ExecutionError::OffsetInvalidIndex(
                                    module.get_value(index).clone(),
                                    index_val.clone(),
                                    bound
                                ))
                            }
                        },
                        _ => Err(ExecutionError::TypeMismatch(
                                module.get_value(index).clone(),
                                index_val.clone()))
                    }
                })
                .collect::<Result<_, _>>()?;
            
            // compute accumulated offset
            let last_dim_subdim = [Some(1usize)];
            let total_offset: usize = indices
                .into_iter().zip(inner.bounds.iter().cloned().skip(1).chain(last_dim_subdim.into_iter()))
                .fold(0usize, | acc, (index, next_dim_bound) | {
                    acc + index * next_dim_bound.expect("expected bounded dimension in `Offset`")
                });
            
            let memory_object = match base_addr_val {
                Val::Pointer(memory_object) => Ok(memory_object.clone()),
                _ => Err(ExecutionError::TypeMismatch(base_addr_value.clone(), base_addr_val.clone()))
            }?;
            MemoryObject::try_from_offset(&memory_object, total_offset)
                .map_or_else(| | Err(ExecutionError::OffsetExceedMemoryRegion(value_data.clone())),
                | memory_obj | Ok(Val::Pointer(memory_obj)))
        },
        ValueKind::FnCall(inner) => {

            let args_val = inner.args
                .iter().cloned()
                .map(| arg_ref | env.get_val(arg_ref).clone())
                .collect::<Vec<_>>();

            let func_ref = module.get_function_ref(&inner.callee);
            run_on_function(
                env,
                module,
                func_ref,
                args_val)
        },
        ValueKind::Alloca(inner) => {
            let function = env.get_top_function(module);
            let memory_object = MemoryObject {
                function: function.name.clone(),
                base: value,
                offset_within: 0,
                size: inner.num_elements
            };
            env.initialize_memory(value, inner.num_elements);
            Ok(Val::Pointer(memory_object))
        },
        ValueKind::Load(inner) => {
            let addr_value = module.get_value(inner.addr);
            let addr_val = env.get_val(inner.addr);
            let memory_object = match addr_val {
                Val::Pointer(memory_object) => Ok(memory_object.clone()),
                _ => Err(ExecutionError::TypeMismatch(addr_value.clone(), addr_val.clone()))
            }?;

            let whole_object = env.get_memory(memory_object.base);
            Ok(whole_object[memory_object.offset_within].clone())
        },
        ValueKind::Store(inner) => {
            let addr_value = module.get_value(inner.addr);
            let addr_val = env.get_val(inner.addr);
            let memory_object = match addr_val {
                Val::Pointer(memory_object) => Ok(memory_object.clone()),
                _ => Err(ExecutionError::TypeMismatch(addr_value.clone(), addr_val.clone()))
            }?;

            let value_stored = env.get_val(inner.value).clone();
            let whole_object = env.get_memory_mut(memory_object.base);
            whole_object[memory_object.offset_within] = value_stored;
            Ok(Val::Unit)
        }
        _ => Err(ExecutionError::NotImplemented(String::from("Expected Instruction")))
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
        let val = single_step(env, module, instr)?;
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
    env.frames.push(Frame::new(function));
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
    env.frames.pop();
    Ok(bb_exit_val)

}


pub fn run_on_module(
    env: &mut ProgramEnv,
    module: &Module,
    entry_fn: &str,
    args: Vec<Val>
) -> Result<Val, ExecutionError> {
    let mut global_frame = env.get_global_frame_mut();
    // set all constant value
    module.value_ctx
        .iter()
        .for_each(| (value, value_data) | {
            match &value_data.kind {
                ValueKind::ConstantInt(inner) =>
                    global_frame.set_local_val(value, Val::Integer(inner.value)),
                ValueKind::ConstantBool(inner) =>
                    global_frame.set_local_val(value, Val::Bool(inner.value)),
                ValueKind::ConstantUnit(_) =>
                    global_frame.set_local_val(value, Val::Unit),
                _ => None,
            };
        });

    let function = module.get_function_ref(entry_fn);
    run_on_function(env, module, function, args)
}