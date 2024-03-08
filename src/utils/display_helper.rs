use crate::ir::structures::*;

pub struct DisplayWithContext<'a, NotDisplayable, Context> {
    pub item: &'a NotDisplayable,
    pub context: &'a Context
}

pub trait FromNotDisplayable<'a, T, C>: Sized {
    fn wrap_context(&'a self, context: &'a C) -> DisplayWithContext<Self, C> {
        DisplayWithContext { item: self, context }
    }
}

impl<'a> FromNotDisplayable<'a, Value, Module> for Value {}
impl<'a> FromNotDisplayable<'a, BasicBlock, Module> for BasicBlock {}
impl<'a> FromNotDisplayable<'a, Function, Module> for Function {}