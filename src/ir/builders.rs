
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use slotmap::SlotMap;

use crate::utils::unique_name::UniqueName;
use super::types::{Type, TypeKind};
use super::{structures::*, values};


struct FunctionEmitState {
    /* local variabels */
    local_string_value_map: HashMap<String, ValueRef>,
    local_string_bb_map: HashMap<String, BlockRef>,

    current_function: FunctionRef,
    position: Option<BlockRef>,
    namer: UniqueName,
}

impl FunctionEmitState {
    pub fn new(func: FunctionRef) -> FunctionEmitState {
        FunctionEmitState {
            local_string_bb_map: HashMap::new(),
            local_string_value_map: HashMap::new(),
            current_function: func,
            position: None,
            namer: UniqueName::new(),
        }
    }
}


pub struct IRBuilder {
    pub module: Module,
    func: Option<FunctionEmitState>,

    /* global variables */
    global_string_value_map: HashMap<String, ValueRef>,
}


impl IRBuilder {
    pub fn new() -> IRBuilder {
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

    pub fn get_value_ref(&self, name: &str) -> Option<ValueRef> {
        self.func
            .as_ref()
            .unwrap()
            .local_string_value_map
            .get(name)
            .cloned()
    }

    pub fn get_block_ref(&self, name: &str) -> Option<BlockRef> {
        self.func
            .as_ref()
            .unwrap()
            .local_string_bb_map
            .get(name)
            .cloned()
    }

    fn get_current_function_data_mut(&mut self) -> &mut Function {
        let state = self.func
            .as_mut()
            .expect("IR builder has no working function");
        self.module
            .get_function_mut(state.current_function)
    }

    fn get_current_block_data_mut(&mut self) -> &mut BasicBlock {
        let state = self.func
            .as_mut()
            .expect("IR builder has no working function");
        let working_bb = state
            .position
            .expect("IR builder has no working basic block");
        let current_function = self.module
            .func_ctx
            .get_mut(state.current_function)
            .unwrap();
        current_function.blocks_ctx
            .get_mut(working_bb)
            .unwrap()
    }

    /// This function is only for create a placeholder basic block,
    /// which is used as the destinations of terminator.
    /// The created basic block will NOT be appended to basic block list,
    /// but insert into symbol map as a placeholder.
    /// The actual basic block will be parsed later and fixup.
    pub fn get_or_insert_placeholder_block_ref(&mut self, name: &str) -> BlockRef {
        let possible_bb = self.func
            .as_ref()
            .unwrap()
            .local_string_bb_map
            .get(name)
            .cloned();
        match possible_bb {
            Some(bb_ref) => bb_ref,
            None => {
                let mut placeholder_bb = BasicBlock::new();
                placeholder_bb.set_name(Some(name.into()));
                // only insert symbol not push into the function structure.
                let curruent_function = self.get_current_function_data_mut();
                let handler = curruent_function.insert_dangling_basic_block(placeholder_bb);
                self.func
                    .as_mut()
                    .expect("builder has no working function")
                    .local_string_bb_map
                    .insert(name.into(), handler);
                handler
            }
        }

    }

    pub fn insert_literal_value(&mut self, value: Value) -> ValueRef {
        self.module.insert_value(value)
    }

    fn insert_value(&mut self, value: Value) -> ValueRef {
        self.module.insert_value(value)
    }

    fn append_basic_block(&mut self, bb: BasicBlock) -> BlockRef {
        let current_function = self.get_current_function_data_mut();
        current_function.append_basic_block(bb)
    }

    /// get the handler of value and update local symbol value map
    fn insert_local_value_symbol(&mut self, value: Value) -> ValueRef {
        let value_name = value.name.clone();
        let handler = self.insert_value(value);
        match value_name {
            Some(name) => {
                self.func
                    .as_mut()
                    .expect("builder has no working function")
                    .local_string_value_map
                    .insert(name, handler);
                handler
            }
            None => handler
        }
    }

    /// get the handler of basic block and update local symbol value map
    fn append_basic_block_symbol(&mut self, bb: BasicBlock) -> BlockRef {
        let bb_name = bb.name.clone();
        let handler = self.append_basic_block(bb.clone());
        match bb_name {
            Some(name) => {
                self.func
                    .as_mut()
                    .expect("builder has no working function")
                    .local_string_bb_map
                    .insert(name, handler);
                handler
            },
            None => handler
        }
    }

    fn insert_instruction_symbol(&mut self, instr: Value) -> ValueRef {
        let handler = self.insert_local_value_symbol(instr);
        let working_bb = self.get_current_block_data_mut();
        working_bb.insert_instr_before_terminator(handler);
        handler
    }

    pub fn get_value(&self, value_ref: ValueRef) -> Value {
        self.module.get_value(value_ref).clone()
    }


    pub fn emit_function(&mut self, 
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
            .map(| value | self.insert_value(value.clone()))
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
            name: name.clone(),
            args: args_value_ref,
            is_external,
            blocks: Vec::new(),
            blocks_ctx: SlotMap::with_key()
        };

        let func_ref = self.module.append_function(function);

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

    /// create an empty basic block or push back created dangling basic block created.
    pub fn emit_basic_block(&mut self, name: Option<String>) -> BlockRef {

        match name {
            Some(bb_name) => {
                if let Some(dangling_bb_ref) = self.get_block_ref(&bb_name) {
                    println!("find dangling basic block {}\n", bb_name);
                    let current_function = self.get_current_function_data_mut();
                    current_function.append_back_dangling_basic_block(dangling_bb_ref);
                    dangling_bb_ref
                } else {
                    let current_function = self.get_current_function_data_mut();
                    let mut new_bb = BasicBlock::new();
                    new_bb.set_name(Some(bb_name));
                    current_function.append_basic_block(new_bb)
                }
            },
            None => {
                let current_function = self.get_current_function_data_mut();
                let mut new_bb = BasicBlock::new();
                new_bb.set_name(name);
                current_function.append_basic_block(new_bb)
            }
        }
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

        let mut binexpr = values::Binary::new(result_ty, op, lhs, rhs);
        binexpr.set_name(inner_name);
        self.insert_instruction_symbol(binexpr)

    }

    pub fn fixup_terminator_jump(&mut self, dest: BlockRef) {
        let state = self.func
            .as_mut()
            .unwrap();
        let working_bb = state.position.unwrap();
        let current_function = self.module
            .func_ctx
            .get_mut(state.current_function)
            .unwrap();
        current_function.blocks_ctx
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
        let cond_value = self.module
            .value_ctx
            .get(cond)
            .unwrap();

        assert!(cond_value.ty.is_i1_type(),
                "expect condition value i1 type in branch terminator, but found type `{}`",
                cond_value.ty.clone());
        let current_function = self.module
            .func_ctx
            .get_mut(state.current_function)
            .unwrap();
        current_function.blocks_ctx
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

        let ret_value = self.module
            .value_ctx
            .get(return_value)
            .unwrap();
        let current_function = self.module
            .func_ctx
            .get(state.current_function)
            .unwrap();
        let expected_ret_ty: &TypeKind = &current_function.ty;
        match expected_ret_ty {
            TypeKind::Function(_, ret_ty) =>
                assert!(ret_value.ty.eq(&ret_ty),
                        "expected return value {} type, but found {} type",
                        ret_ty.clone(),
                        ret_value.ty.clone()),
            _ => ()
        }

        let working_bb = state.position.unwrap();
        let current_function = self.module
            .func_ctx
            .get_mut(state.current_function)
            .unwrap();
        current_function.blocks_ctx
            .get_mut(working_bb)
            .unwrap()
            .set_terminator(
                values::Return::new_value(return_value)
            )
    }
}