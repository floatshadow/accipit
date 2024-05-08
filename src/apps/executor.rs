use std::fmt;
use std::char;
use std::str::FromStr;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};

use crate::ir::{
    values,
    structures::*
};
use crate::utils::{
    to_c_str,
    display_helper::*
};

use slotmap::SecondaryMap;
use libc::*;
use colored::Colorize;

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
    InternalError(String),
    FunctionNumArgumentMismatch(String, Vec<Val>),
    ReturnDanglingPointer(Value),
    LexerError,
    ParseError
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", "error: ".red().bold())?;
        match self {
            Self::SymbolNotFound(s) =>
                write!(f, "'{}' symbol not found", s.bold()),
            Self::TypeMismatch(value, val) =>
                write!(f, "the type of value '{}' is incompatiable with input '{}'",
                        value.to_string().bold(), val.to_string().bold()),
            Self::InvalidInputArguments(s) =>
                write!(f, "invalid input argument '{}'", s.bold()),
            Self::OffsetInvalidIndex(offset, index, bound) => {
                match bound {
                    Some(bound) =>
                        write!(f, "in offset '{}', index ['{}' < '{}'] is invalid",
                                offset.to_string().bold(), index.to_string().bold(), bound.to_string().bold()),
                    None =>
                        write!(f, "in offset '{}', index ['{}' < '{}'] is invalid",
                                offset.to_string().bold(), index.to_string().bold(), "none".bold()),
                }
            },
            Self::OffsetExceedMemoryRegion(offset) =>
                write!(f, "offset '{}' exceeded memory bound", offset.to_string().bold()),
            Self::InternalError(s) =>
                write!(f, "internal function error {}", s),
            Self::FunctionNumArgumentMismatch(name, param) => {
                writeln!(f, "function '{}' get inconsistent number of arguments with its parameter list", name.bold())?;
                write!(f, "    input params '[{}]'", param.iter().format_with(", ", | elem, f | f(&format_args!("{}", elem.to_string().bold()))))
            },
            Self::ReturnDanglingPointer(value) => {
                write!(f, "try to returns a dangling pointer '{}'", value.to_string().bold())
            },
            Self::LexerError => write!(f, "lexing error"),
            Self::ParseError => write!(f, "parsing error"),
            _ => unreachable!()
        }
    }
}

/// Trace the source of pointer values,
/// including function parameters, local allocas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryObject {
    pub frame_index: usize,
    /// Function scope of the object.
    pub function: FunctionRef,
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
                frame_index: base.frame_index,
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
            .map_err(| _ | ExecutionError::InvalidInputArguments(format!("{}", s.bold())))?;

        Ok(Val::Integer(int_value))
    }
}

impl fmt::Display for Val {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Val::Unit => Ok(()),
            Val::Integer(inner) => write!(f, "{}", inner),
            Val::Bool(inner) => write!(f, "{}", inner),
            Val::Pointer(inner) => write!(f, "<inner pointer>: {:?}", inner),
            Val::Function(name) => write!(f, "function: {}", name),
            Val::Undefined => write!(f, "<undefined>")
        }
    }
}

impl<'a> fmt::Display for DisplayWithContext<'a, Val, Module> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = self.item;
        let module = self.context;
        match val {
            Val::Unit => write!(f, "()"),
            Val::Integer(inner) => write!(f, "{}", inner),
            Val::Bool(inner) => write!(f, "{}", inner),
            Val::Pointer(inner) =>
                write!(f, "<inner pointer>: [stack_depth: {}, function: {}, base_value: {}, offset: {}, region_size: {}]",
                        inner.frame_index, module.get_function(inner.function).name, module.get_value(inner.base), inner.offset_within, inner.size),
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

#[derive(Debug, Clone)]
pub struct Frame {
    pub frame_val_env: SecondaryMap<ValueRef, Val>,
    pub local_allocas: HashSet<ValueRef>,
    pub working_function: FunctionRef
}

impl Frame {
    pub fn new(working_function: FunctionRef) -> Frame {
        Frame {
            frame_val_env: SecondaryMap::new(),
            local_allocas: HashSet::new(),
            working_function
        }
    }

    pub fn set_local_val(&mut self, value: ValueRef, val: Val) -> Option<Val> {
        self.frame_val_env.insert(value, val) 
    }

    pub fn get_local_val(&self, value: ValueRef) -> Option<&Val> {
        self.frame_val_env.get(value)
    }
}

#[derive(Debug, Clone)]
pub struct ProgramEnv {
    /// current working basic block.
    pub position: Option<BlockRef>,
    /// program counter.
    pub program_counter: Option<ValueRef>,
    /// memory, managed as global state.
    pub memory: HashMap<(ValueRef, usize), Vec<Val>>,
    /// global values.
    pub global_val: SecondaryMap<ValueRef, Val>,
    /// function frames.
    pub frames: Vec<Frame>
}

impl ProgramEnv {
    pub fn get_top_frame(&self) -> Option<&Frame> {
        self.frames.last()
    }

    pub fn get_num_frames(&self) -> usize {
        self.frames.len()
    }

    pub fn prologue(&mut self, function: FunctionRef) {
        self.frames.push(Frame::new(function))
    }

    pub fn epilogue(&mut self) {
        let exit_frame = self.frames.last()
            .expect("no active frame frame");
        let frame_index = self.get_num_frames();
        for allocas in exit_frame.local_allocas.iter().cloned() {
            let key_info = (allocas, frame_index);
            self.memory.remove(&key_info);
        }
        self.frames.pop();
    }

    // non persistent mutable update.
    pub fn set_value_binding(&mut self, value: ValueRef, val: Val) -> Option<Val> {
        let top_frame = self.frames
            .last_mut()
            .expect("no active function frame");
        top_frame.set_local_val(value, val)
    }

    pub fn get_val(&self, value: ValueRef) -> &Val {
        let top_frame = self.frames
            .last()
            .expect("no active function frame");
        top_frame
            .get_local_val(value)
            .or_else(| | self.global_val.get(value))
            .expect("value does not exist")
    }

    pub fn get_memory(&self, ptr: ValueRef, frame_index: usize) -> &Vec<Val> {
        let key_info = (ptr, frame_index);
        self.memory
            .get(&key_info)
            .expect("try to load from a memory region which does not exist")
    }

    pub fn get_memory_mut(&mut self, ptr: ValueRef, frame_index: usize) -> &mut Vec<Val> {
        let key_info = (ptr, frame_index);
        self.memory
            .get_mut(&key_info)
            .expect("try to store into a memory region which does not exist")
    }

    pub fn initialize_memory(&mut self, ptr: ValueRef, size: usize) {
        let unitialized_object = vec![Val::Undefined; size];
        let frame_index = self.get_num_frames();
        self.memory.insert((ptr, frame_index), unitialized_object);
    }
}

impl ProgramEnv {
    pub fn new() -> ProgramEnv {
        ProgramEnv {
            position: None,
            program_counter: None,
            memory: HashMap::new(),
            global_val: SecondaryMap::new(),
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
            // runtime IO
            match inner.callee.as_str() {
                "getint" => {
                    let mut value: i32 = 0;
                    unsafe {
                        assert!(
                            scanf(to_c_str("%d").as_ptr(), &mut value as *mut i32) == 1,
                            "'getint' expect a 'int' input"
                        );
                    }
                    Ok(Val::Integer(value))
                },
                "getch" => {
                    let mut character: c_char = 0;
                    unsafe {
                        assert!(
                            scanf(to_c_str("%c").as_ptr(), &mut character as *mut c_char) == 1,
                            "'getch' expect a 'char' input"
                        );
                    }
                    Ok(Val::Integer(character as i32))
                },
                "getarray" => {
                    assert!(args_val.len() == 1, "'{}' expect 1 argument", "getarray".bold());
                    let addr = args_val[0].clone();
                    match addr {
                        Val::Pointer(inner) => {
                            let mut n: i32 = 0;
                            unsafe {
                                assert!(
                                    scanf(to_c_str("%d").as_ptr(), &mut n as *mut i32) == 1,
                                    "'getarray' expect a 'int' input as array size"
                                );
                            }
                            let buffer = env.get_memory_mut(inner.base, inner.frame_index);
                            assert!(n >= 0, "'{}' expect a non-negative array size", "getarray".bold());
                            for i in 0..n {
                                let mut val: i32 = 0;
                                unsafe {
                                    assert!(
                                        scanf(to_c_str("%d").as_ptr(), &mut val as *mut i32) == 1,
                                        "expect a 'int' input as array element"
                                    );
                                }
                                assert!(inner.offset_within + (i as usize) < inner.size,
                                        "'{}' access memory out of bounds", "getarray".bold()
                                );
                                buffer[inner.offset_within + i as usize] = Val::Integer(val);
                            }
                            Ok(Val::Integer(n))
                        },
                        _ => Err(ExecutionError::InternalError(format!("'{}' accepts 1 pointer type argument only", "getarray".bold())))
                    }
                },
                "putint" => {
                    assert!(args_val.len() == 1, "'{}' expect 1 argument", "putint".bold());
                    let output_value = args_val[0].clone();
                    match output_value {
                        Val::Integer(inner) => {
                            print!("{}", inner);
                            use std::io::Write;
                            std::io::stdout().flush().expect("unable to flush output stream after 'putint'");
                            Ok(Val::Unit)
                        },
                        _ => Err(ExecutionError::InternalError(format!("'{}' accepts 1 integer type argument only", "putint".bold())))
                    }
                },
                "putch" => {
                    assert!(args_val.len() == 1, "'{}' expect 1 argument", "putch".bold());
                    let output_value = args_val[0].clone();
                    match output_value {
                        Val::Integer(inner) => {
                            print!("{}", char::from_u32(inner as u32).expect("ilegal char value in 'putch'"));
                            use std::io::Write;
                            std::io::stdout().flush().expect("unable to flush output stream after 'putch'");
                            Ok(Val::Unit)
                        },
                        _ => Err(ExecutionError::InternalError(format!("'{}' accepts 1 integer type argument only","putch".bold())))
                    }
                },
                "putarray" => {
                    assert!(args_val.len() == 2, "'{}' expect 2 argument", "putarray".bold());
                    let num = match &args_val[0] {
                        Val::Integer(inner) => Ok(inner.clone()),
                        _ => Err(ExecutionError::InternalError(format!("'{}' expect integer type argument as array size", "putarray".bold())))
                    }?;
                    print!("{}:", num);
                    let addr = args_val[1].clone();
                    match addr {
                        Val::Pointer(inner) => {
                            let buffer = env.get_memory(inner.base, inner.frame_index);
                            assert!(num >= 0, "'{}', expect a non-negative array size", "putarray".bold());
                            for i in 0..num {
                                assert!(inner.offset_within + (i as usize) < inner.size,
                                        "'{}' access memory out of bounds", "putarray".bold()
                                );
                                let load_val =  buffer[inner.offset_within + i as usize].clone();
                                match load_val {
                                    Val::Integer(inner) => print!(" {}", inner),
                                    _ => panic!("'{}' accept a non integer array", "putarray".bold())
                                }
                            }
                            use std::io::Write;
                            std::io::stdout().flush().expect("unable to flush output stream after 'putarray'");
                            Ok(Val::Unit)
                        },
                        _ => Err(ExecutionError::InternalError(format!("'{}' accepts 1 pointer type argument only", "getarray".bold())))
                    }
                },
                "starttime" | "stoptime" => {
                    print!("{} '{}' and '{}' do nothing in interpreter",
                            "Warning: ".yellow(), "starttime".bold(), "stoptime".bold());
                    use std::io::Write;
                    std::io::stdout().flush().expect("unable to flush output stream after 'start/stoptime'");
                    Ok(Val::Unit)
                },
                _ => {
                    let func_ref = module.get_function_ref(&inner.callee);
                    run_on_function(
                        env,
                        module,
                        func_ref,
                        args_val)
                }
            }
        },
        ValueKind::Alloca(inner) => {
            let frame = env.get_top_frame()
                .expect("no active function frame");
            let memory_object = MemoryObject {
                frame_index: env.get_num_frames(),
                function: frame.working_function,
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
            let ptr = match addr_val {
                Val::Pointer(inner) => Ok(inner.clone()),
                _ => Err(ExecutionError::TypeMismatch(addr_value.clone(), addr_val.clone()))
            }?;

            let region = env.get_memory(ptr.base, ptr.frame_index);
            Ok(region[ptr.offset_within].clone())
        },
        ValueKind::Store(inner) => {
            let addr_value = module.get_value(inner.addr);
            let addr_val = env.get_val(inner.addr);
            let ptr = match addr_val {
                Val::Pointer(inner) => Ok(inner.clone()),
                _ => Err(ExecutionError::TypeMismatch(addr_value.clone(), addr_val.clone()))
            }?;
            let value_stored = env.get_val(inner.value).clone();
            let region = env.get_memory_mut(ptr.base, ptr.frame_index);
            region[ptr.offset_within] = value_stored;
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
                Val::Integer(num) if num.clone() != 0 => {
                    env.position = Some(inner.true_label);
                    Ok(Val::Unit)
                },
                Val::Integer(num) if num.clone() == 0 => {
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
            // finish running
            env.position = None;
            let ret_val = env.get_val(inner.value);
            // check dangling pointer
            match &ret_val {
                Val::Pointer(mem) if mem.frame_index > env.frames.len() => {
                    Err(ExecutionError::ReturnDanglingPointer(
                        module.get_value(inner.value).clone())
                    )
                }
                _ => Ok(ret_val.clone())
            }
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
        env.set_value_binding(instr, val);
    };
    single_step_terminator(env, module, &block.terminator)
}

pub fn run_on_function(
    env: &mut ProgramEnv,
    module: &Module,
    function: FunctionRef,
    args: Vec<Val>
) -> Result<Val, ExecutionError> {
    env.prologue(function);
    let function = module.get_function(function);
    // set args values
    if function.args.len() != args.len() {
        return Err(ExecutionError::FunctionNumArgumentMismatch(function.name.clone(), args));
    }
    let params: Vec<Val> = args
        .into_iter().zip(function.args.iter().cloned())
        .map( | (arg, param) | {
            let value = module.get_value(param);
            arg.matches_value(value)
        } )
        .collect::<Result<_, _>>()?;

    params
        .into_iter().zip(function.args.iter().cloned())
        .for_each(| (val, value) | { env.set_value_binding(value, val); } );

    let entry_bb = function.blocks[0];
    env.position = Some(entry_bb);
    let mut bb_exit_val = Val::Undefined;

    while let Some(current_bb) = env.position {
        env.position = Some(current_bb);
        let block = function.get_basic_block(current_bb);
        bb_exit_val = run_on_basicblock(env, module, block)?;
    };
    env.epilogue();
    Ok(bb_exit_val)

}


pub fn run_on_module(
    env: &mut ProgramEnv,
    module: &Module,
    entry_fn: &str,
    args: Vec<Val>
) -> Result<Val, ExecutionError> {
    // FIXME, insert a phantom function as the global 'frame'.
    use crate::ir::types::Type;
    use slotmap::SlotMap;
    let phantom_function = Function {
        ty: Type::get_function( vec![], Type::get_unit()),
        name: "<global frame>".to_string(),
        args: vec![],
        is_external: false,
        blocks: vec![],
        blocks_ctx: SlotMap::with_key()
    };
    let mut phantom_function_ctx: SlotMap<FunctionRef, Function> = SlotMap::with_key();
    let phantom_function_ref = phantom_function_ctx.insert(phantom_function);
    // set all constant value
    module.value_ctx
        .iter()
        .for_each(| (value, value_data) | {
            match &value_data.kind {
                ValueKind::ConstantInt(inner) => {
                    env.global_val.insert(value, Val::Integer(inner.value));
                },
                ValueKind::ConstantBool(inner) => {
                    env.global_val.insert(value, Val::Bool(inner.value));
                },
                ValueKind::ConstantUnit(_) => {
                    env.global_val.insert(value, Val::Unit);
                },
                ValueKind::GlobalVar(inner) => {
                    env.global_val.insert(value, Val::Pointer(MemoryObject {
                        frame_index: 0,
                        function: phantom_function_ref,
                        base: value,
                        offset_within: 0,
                        size: inner.size
                    }));
                    // allocate memory
                    env.initialize_memory(value, inner.size);
                },
                _ => (),
            };
        });

    let function = module.get_function_ref(entry_fn);
    run_on_function(env, module, function, args)
}