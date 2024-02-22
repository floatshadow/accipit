use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use std::ops::Deref;
use std::collections::HashMap;
use std::sync::Arc;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeKind {
    Int32,
    Int1,
    Unit,
    Pointer(Type),
    OpaquePtr,
    Function(Vec<Type>, Type)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Type(Rc<TypeKind>);

impl Deref for Type {
    type Target = TypeKind;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
} 


impl fmt::Display for TypeKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TypeKind::Int32 => write!(f, "i32"),
            TypeKind::Int1 => write!(f, "i1"),
            TypeKind::Unit => write!(f, "()"),
            TypeKind::Pointer(base_type) => write!(f, "*{}", base_type),
            TypeKind::OpaquePtr => write!(f, "ptr"),
            TypeKind::Function(params_type, res_type) => {
                for param_type in params_type.iter() {
                    write!(f, "{} ->", param_type)?
                };
                write!(f, "{}", res_type)
            }
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


impl Type {
    thread_local! {
        static POOL: RefCell<HashMap<TypeKind, Type>> = RefCell::new(HashMap::new());
    }

    pub fn get(kind: TypeKind) -> Type {
        Self::POOL.with( |pool| {
            pool.
                borrow_mut().
                entry(kind.clone()).
                or_insert(Type(Rc::new(kind.clone()))).
                clone()
        })
    }

    pub fn get_i32() -> Type {
        Type::get(TypeKind::Int32)
    }

    pub fn get_i1() -> Type {
        Type::get(TypeKind::Int1)
    }

    pub fn get_unit() -> Type {
        Type::get(TypeKind::Unit)
    }

    pub fn get_pointer(base_ty: Type) -> Type {
        Type::get(TypeKind::Pointer(base_ty))
    }

    pub fn get_opaque_pointer() -> Type {
        Type::get(TypeKind::OpaquePtr)
    }

    pub fn get_function(params: Vec<Type>, ret: Type) -> Type {
        Type::get(TypeKind::Function(params, ret))
    }

    pub fn is_integer_type(&self) -> bool {
        matches!(self.0.as_ref(), TypeKind::Int32 | TypeKind::Int1)
    }

    pub fn is_unit_type(&self) -> bool {
        matches!(self.0.as_ref(), TypeKind::Unit)
    }

    pub fn is_pointer_type(&self) -> bool {
        matches!(self.0.as_ref(), TypeKind::Pointer(..) | TypeKind::OpaquePtr)
    }

    pub fn is_function_type(&self) -> bool {
        matches!(self.0.as_ref(), TypeKind::Function(..))
    }
}