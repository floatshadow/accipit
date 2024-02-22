use std::fmt;

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
    Mod,
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

#[derive(Debug, Clone)]
pub struct Binary {
    pub op: BinaryOp,
    pub lhs: ValueRef,
    pub rhs: ValueRef
}

#[derive(Debug, Clone)]
pub struct ConstantInt {
    pub value: i64
}

/* Treat the parameters of function as `Value`.
 */
#[derive(Debug, Clone)]
pub struct Argument;

#[derive(Debug, Clone)]
pub struct ConstantNullPtr;

#[derive(Debug, Clone)]
pub struct ConstantUnit;

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

#[derive(Debug, Clone)]
pub struct Branch {
    pub cond: ValueRef,
    pub true_label: BlockRef,
    pub false_label: BlockRef,
}

#[derive(Debug, Clone)]
pub struct Return {
    pub value: ValueRef
}