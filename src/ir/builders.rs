
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
    module: Module,
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
        ret_ty: Type
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
    pub fn emit_basic_block(&mut self) {
        let state = self.func
            .as_mut()
            .expect("builder has no working function");
        let new_bb = BasicBlock::new();
        let new_bb_ref = state.current_function.blocks_ctx.insert(new_bb);
        state.current_function.blocks.push(new_bb_ref);
    }

}