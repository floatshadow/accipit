
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use slotmap::SlotMap;

use crate::utils::unique_name::UniqueName;
use super::types::Type;
use super::{structures::*, values};


struct FunctionEmitState<'a> {
    /* local variabels */
    local_string_value_map: HashMap<String, ValueRef>,
    local_string_bb_map: HashMap<String, BlockRef>,

    current_function: &'a mut Function,
    position: Option<BlockRef>,
    namer: UniqueName,
}

impl<'a> FunctionEmitState<'a> {
    pub fn new(func: &mut Function) -> FunctionEmitState {
        FunctionEmitState {
            local_string_bb_map: HashMap::new(),
            local_string_value_map: HashMap::new(),
            current_function: func,
            position: None,
            namer: UniqueName::new(),
        }
    }
}


pub struct IRBuilder<'a> {
    pub module: Module,
    func: Option<FunctionEmitState<'a>>,

    /* global variables */
    global_string_value_map: HashMap<String, ValueRef>,
}


impl<'a> IRBuilder<'a> {
    pub fn new() -> IRBuilder<'a> {
        IRBuilder {
            module: Module::new(),
            func: None,
            global_string_value_map: HashMap::new(),
        }
    }

    fn get_unique_name(&mut self, base: &Option<String>) -> String {
        let namer = &mut self.func.
            as_mut().
            unwrap().
            namer;
        match base.as_ref() {
            Some(given_name) => namer. next_name(&given_name),
            None => namer.next_anonymous_name()
        }
    }

    fn get_value_ref(&self, name: String) -> ValueRef {
        self.func
            .as_ref()
            .unwrap()
            .local_string_value_map
            .get(&name)
            .unwrap()
            .clone()
    }

    fn get_block_ref(&self, name: String) -> BlockRef {
        self.func
            .as_ref()
            .unwrap()
            .local_string_bb_map
            .get(&name)
            .unwrap()
            .clone()
    }

    /* This function is only for create a phantom basic block,
     * which is used as the destinations of terminator.
     * The actual basic block will be parsed later and fixup.
     */
    pub fn get_or_insert_block(&mut self, name: String) -> BlockRef {
        let possible_bb = self.func
            .as_ref()
            .unwrap()
            .local_string_bb_map
            .get(&name);
        match possible_bb {
            Some(bb_ref) => bb_ref.clone(),
            None => {
                let mut phantom_bb = BasicBlock::new();
                phantom_bb.set_name(Some(name));
                self.insert_basic_block_symbol(phantom_bb)
            }
        }

    }

    fn insert_instruction(&mut self, value_ref: ValueRef) {
        let state = self.func
            .as_mut()
            .unwrap();
        let working_bb = state.position.unwrap();
        state.current_function.blocks_ctx
            .get_mut(working_bb)
            .unwrap()
            .instrs
            .push(value_ref);
    }

    /* get the handler of value and update local symbol value map  */
    fn insert_local_symbol(&mut self, value: Value) -> ValueRef {
        let handler = self.insert_value_inner(value.clone());
        match value.name {
            Some(name) =>
                self.func
                    .as_mut()
                    .expect("builder has no working function")
                    .local_string_value_map
                    .insert(name, handler)
                    .expect("local symbol duplicated name"),
            None => handler
        }
    }

    fn insert_basic_block_symbol(&mut self, bb: BasicBlock) -> BlockRef {
        let handler = self.insert_basic_block_inner(bb.clone());
        match bb.name {
            Some(name) =>
                self.func
                    .as_mut()
                    .expect("builder has no working function")
                    .local_string_bb_map
                    .insert(name, handler)
                    .expect("basic block name duplicated"),
            None => handler
        }
    }

    fn insert_value_inner(&mut self, value: Value) -> ValueRef {
        self.module
            .value_ctx
            .insert(value)
    }

    fn get_value(&self, value_ref: ValueRef) -> Value {
        self.module
            .value_ctx
            .get(value_ref)
            .unwrap()
            .clone()
    }

    fn insert_basic_block_inner(&mut self, bb: BasicBlock) -> BlockRef {
        self.func
            .as_mut()
            .unwrap()
            .current_function
            .blocks_ctx
            .insert(bb)
    }

    pub fn emit_function(&'a mut self, 
        name: String, 
        params_ty: Vec<(Option<String>, Type)>, 
        ret_ty: Type,
        is_external: bool
    )  {
        let (params, params_ty): (Vec<Value>, Vec<Type>) = params_ty
            .into_iter()
            .enumerate()
            .map(| (i, (param_name, ty)) | {
                let arg_value = 
                    values::Argument::new_value_with_name(i, ty.clone(), param_name);
                (arg_value, ty)
            })
            .unzip();
        
        let args_value_ref = params
            .iter()
            .map(| value | self.insert_value_inner(value.clone()))
            .collect::<Vec<_>>();

        let local_name_ctx = args_value_ref
            .iter()
            .map( | value_ref | {
                let value = self.get_value(value_ref.clone());
                (value.name, value_ref.clone())
            })
            .filter_map( | (name, value_ref) | {
                match name {
                    Some(name) => Some((name, value_ref)),
                    None => None
                }
            })
            .collect::<HashMap<_, _>>();
            
        let func_ty = Type::get_function(params_ty, ret_ty);
        let function = Function {
            ty: func_ty,
            name,
            args: args_value_ref,
            is_external,
            blocks: Vec::new(),
            blocks_ctx: SlotMap::with_key()
        };

        let func_ref = self.module
            .func_ctx
            .entry(function.name.clone())
            .or_insert(function);

        self.func = Some(FunctionEmitState {
            local_string_value_map: local_name_ctx,
            local_string_bb_map: HashMap::new(),
            current_function: func_ref,

            position: None,
            namer: UniqueName::new()
        });

        
        
    }

    pub fn set_insert_point(&mut self, bb: BlockRef) {
        let state = self.func
            .as_mut()
            .expect("builder has no working function");
        state.position = Some(bb);
    }

    /* create an empty basic block */
    pub fn emit_basic_block(&mut self, name: Option<String>) -> BlockRef {
        let state = self.func
            .as_mut()
            .expect("builder has no working function");
        let mut new_bb = BasicBlock::new();
        new_bb.set_name(name);
        let new_bb_ref = state.current_function.blocks_ctx.insert(new_bb);
        state.current_function.blocks.push(new_bb_ref);
        new_bb_ref
    }

    pub fn emit_numeric_binary_expr(
        &mut self,
        op: values::BinaryOp,
        name: Option<String>,
        lhs: ValueRef, 
        rhs: ValueRef, 
        annotated_type: Option<Type>
    ) -> ValueRef {
        let lhs_ty = self.module.get_value_type(lhs);
        let rhs_ty = self.module.get_value_type(rhs);
        let inner_name = self.get_unique_name(&name);
        assert!(
            lhs_ty.is_integer_type() && lhs_ty.eq(&rhs_ty),
            "`lhs` and `rhs` should be the same integer type for {}",
            inner_name
        );
        let result_ty = match annotated_type {
            Some(check_ty) => {
                assert!(
                    lhs_ty.eq(&check_ty),
                    "expect type `{}` for `{}`, but found wrong annotation `{}`", 
                    lhs_ty, inner_name, check_ty
                );
                match op {
                    values::BinaryOp::Add | values::BinaryOp::Sub |
                    values::BinaryOp::Mul | values::BinaryOp::Div | values::BinaryOp::Rem |
                    values::BinaryOp::And | values::BinaryOp::Or | values::BinaryOp::Xor =>
                        lhs_ty,
                    values::BinaryOp::Lt | values::BinaryOp::Gt |
                    values::BinaryOp::Le | values::BinaryOp::Ge |
                    values::BinaryOp::Eq | values::BinaryOp::Ne =>
                        Type::get_i1()
                }
            },
            None => lhs_ty
        };

        let handler = self.insert_local_symbol(values::Binary::new(result_ty, op, lhs, rhs));
        self.insert_instruction(handler);
        handler

    }

    pub fn fixup_terminator_jump(&mut self, dest: BlockRef) {
        let state = self.func
            .as_mut()
            .unwrap();
        let working_bb = state.position.unwrap();
        state.current_function.blocks_ctx
            .get_mut(working_bb)
            .unwrap()
            .set_terminator(
                values::Jump::new_value(dest)
            )
    }

    pub fn fixup_terminator_branch(
        &mut self, 
        cond: ValueRef,
        true_label: BlockRef,
        false_label: BlockRef
    ) {
        let state = self.func
            .as_mut()
            .unwrap();
        let working_bb = state.position.unwrap();
        state.current_function.blocks_ctx
            .get_mut(working_bb)
            .unwrap()
            .set_terminator(
                values::Branch::new_value(cond, true_label, false_label)
            )
    }

    pub fn fixup_terminator_return(
        &mut self, 
        return_value: ValueRef
    ) {
        let state = self.func
            .as_mut()
            .unwrap();
        let working_bb = state.position.unwrap();
        state.current_function.blocks_ctx
            .get_mut(working_bb)
            .unwrap()
            .set_terminator(
                values::Return::new_value(return_value)
            )
    }
}