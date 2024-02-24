use std::fmt;

use super::structures::{Terminator, Value, ValueKind};
use super::types::Type;

use super::structures::{
    ValueRef, BlockRef
};



#[derive(Debug, Clone)]
pub struct Function;

#[derive(Debug, Clone)]
pub struct BasicBlock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    /* Numeric Operations */
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Or,
    Xor,
    /* Camparison */
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "add"),
            BinaryOp::Sub => write!(f, "sub"),
            BinaryOp::Mul => write!(f, "mul"),
            BinaryOp::Div => write!(f, "div"),
            BinaryOp::Rem => write!(f, "rem"),
            BinaryOp::And => write!(f, "or"),
            BinaryOp::Or => write!(f, "and"),
            BinaryOp::Xor => write!(f, "xor"),
            BinaryOp::Lt => write!(f, "lt"),
            BinaryOp::Gt => write!(f, "gt"),
            BinaryOp::Le => write!(f, "le"),
            BinaryOp::Ge => write!(f, "ge"),
            BinaryOp::Eq => write!(f, "eq"),
            BinaryOp::Ne => write!(f, "ne")
        }
    }
}

#[derive(Debug, Clone)]
pub struct Binary {
    pub op: BinaryOp,
    pub lhs: ValueRef,
    pub rhs: ValueRef
}

impl Binary {
    pub fn new(ty: Type, op: BinaryOp, lhs: ValueRef, rhs: ValueRef) -> Value {
        Value::new(ty, None, ValueKind::Binary(Self {op, lhs, rhs}))
    }
}

#[derive(Debug, Clone)]
pub struct ConstantInt {
    pub value: i64
}

impl ConstantInt {
    pub fn new_value(value: i64) -> Value {
        Value::new(Type::get_i64(), None, ValueKind::ConstantInt(Self {value}))
    }

    pub fn new_bool_value(value: i8) -> Value {
        let inner_value = i64::from(value);
        Value::new(Type::get_i1(), None, ValueKind::ConstantInt(Self { value: inner_value }))
    }

    pub fn new_true_value() -> Value {
        ConstantInt::new_bool_value(1i8)
    }

    pub fn new_false_value() -> Value {
        ConstantInt::new_bool_value(0i8)
    }
}

/* Treat the parameters of function as `Value`.
 */
#[derive(Debug, Clone)]
pub struct Argument {
    pub index: usize
}

impl Argument {
    pub fn new_value(index: usize, ty: Type) -> Value {
        Value::new(ty, None, ValueKind::Argument(Self {index}))
    }

    pub fn new_value_with_name(index: usize, ty: Type, name: Option<String>) -> Value {
        Value::new(ty, name, ValueKind::Argument(Self { index }))
    }
}

#[derive(Debug, Clone)]
pub struct ConstantNullPtr;

impl ConstantNullPtr {
    pub fn new_value() -> Value {
        Value::new(Type::get_opaque_pointer(), None, ValueKind::ConstantNullPtr(Self))
    }
}

#[derive(Debug, Clone)]
pub struct ConstantUnit;

impl ConstantUnit {
    pub fn new_value() -> Value {
        Value::new(Type::get_unit(), None, ValueKind::ConstantUnit(Self)) 
    }
}

#[derive(Debug, Clone)]
pub struct Offset {
    pub base_addr: ValueRef,
    pub bounds: Vec<Option<usize>>,
    pub index: Vec<ValueRef>,
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    /* For simplicity, we use raw name for the callee function and assume no function overrides.
     * But strictly speaking, `Function`` itself should be treated as a `Value`.
     */
    pub callee: String,
    pub args: Vec<ValueRef>
}

#[derive(Debug, Clone)]
pub struct Alloca {
    pub num_elements: usize
}

#[derive(Debug, Clone)]
pub struct Load {
    pub addr: ValueRef
}

#[derive(Debug, Clone)]
pub struct Store {
    pub addr: ValueRef,
    pub value: ValueRef
}

#[derive(Debug, Clone)]
pub struct Jump {
    pub dest: BlockRef
}

impl Jump {
    pub fn new_value(dest: BlockRef) -> Terminator {
        Terminator::Jump(Self { dest })
    }
}

#[derive(Debug, Clone)]
pub struct Branch {
    pub cond: ValueRef,
    pub true_label: BlockRef,
    pub false_label: BlockRef,
}

impl Branch {
    pub fn new_value(
        cond: ValueRef,
        true_label: BlockRef,
        false_label: BlockRef
    ) -> Terminator {
        Terminator::Branch(Self { cond, true_label, false_label})
    } 
}

#[derive(Debug, Clone)]
pub struct Return {
    pub value: ValueRef
}

impl Return {
    pub fn new_value(value: ValueRef) -> Terminator {
        Terminator::Return(Self { value })
    }
}