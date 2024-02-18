use std::fmt;
use std::rc::Rc;
use std::ops::Deref;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeKind {
    Int,
    Unit,
    Pointer(Type),
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
            TypeKind::Int => write!(f, "Int"),
            TypeKind::Unit => write!(f, "()"),
            TypeKind::Pointer(base_type) => write!(f, "*{}", base_type),
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