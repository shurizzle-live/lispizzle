use std::fmt;

use ecow::EcoString;
use im_rc::Vector;
use rug::Integer;

use crate::{
    util::{print_list_debug, print_list_display},
    Callable, Environment, Lambda, Symbol, Var,
};

#[derive(Clone)]
pub enum Value {
    Unspecified,
    Nil,
    Boolean(bool),
    Character(char),
    Integer(Integer),
    String(EcoString),
    Symbol(Symbol),
    Lambda(Lambda),
    List(Vector<Value>),
    Var(Var),
    Environment(Environment),
}

impl Value {
    #[inline]
    pub fn is_unknown(&self) -> bool {
        matches!(self, Value::Unspecified)
    }

    #[inline]
    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }

    #[inline]
    pub fn is_boolean(&self) -> bool {
        matches!(self, Value::Boolean(_))
    }

    #[inline]
    pub fn is_character(&self) -> bool {
        matches!(self, Value::Character(_))
    }

    #[inline]
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    #[inline]
    pub fn is_symbol(&self) -> bool {
        matches!(self, Value::Symbol(_))
    }

    #[inline]
    pub fn is_lambda(&self) -> bool {
        matches!(self, Value::Lambda(_))
    }

    #[inline]
    pub fn is_list(&self) -> bool {
        matches!(self, Value::List(_))
    }

    #[inline]
    pub fn is_environment(&self) -> bool {
        matches!(self, Value::List(_))
    }

    #[inline]
    pub fn apply(&self, args: Vector<Value>) -> Value {
        match self {
            Self::Lambda(l) => l.call(args),
            _ => Self::Nil,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Unspecified, Self::Unspecified) => true,
            (Self::Nil, Self::Nil) => true,
            (Self::Boolean(l0), Self::Boolean(r0)) => l0 == r0,
            (Self::Character(l0), Self::Character(r0)) => l0 == r0,
            (Self::Integer(l0), Self::Integer(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Symbol(l0), Self::Symbol(r0)) => l0 == r0,
            (Self::Lambda(l0), Self::Lambda(r0)) => l0 == r0,
            (Self::List(l0), Self::List(r0)) => l0 == r0,
            (Self::Var(l0), Self::Var(r0)) => l0 == r0,
            (Self::Environment(l0), Self::Environment(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl Eq for Value {}

impl From<bool> for Value {
    #[inline]
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<char> for Value {
    #[inline]
    fn from(value: char) -> Self {
        Self::Character(value)
    }
}

impl From<Integer> for Value {
    #[inline]
    fn from(value: Integer) -> Self {
        Self::Integer(value)
    }
}

macro_rules! impl_int_into {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl From<$ty> for Value {
                #[inline]
                fn from(value: $ty) -> Self {
                    Integer::from(value).into()
                }
            }

            impl From<&$ty> for Value {
                #[inline]
                fn from(value: &$ty) -> Self {
                    Integer::from(*value).into()
                }
            }

            impl From<&mut $ty> for Value {
                #[inline]
                fn from(value: &mut $ty) -> Self {
                    Integer::from(*value).into()
                }
            }
        )+
    };
}

impl_int_into! {
    i8, u8,
    i32, u32,
    i64, u64,
    i128, u128,
    isize, usize,
}

impl From<EcoString> for Value {
    #[inline]
    fn from(value: EcoString) -> Self {
        Self::String(value)
    }
}

impl From<&str> for Value {
    #[inline]
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}

impl From<String> for Value {
    #[inline]
    fn from(value: String) -> Self {
        Self::String(value.into())
    }
}

macro_rules! impl_into_str {
    ($($ty:ty);+ $(;)?) => {
        $(
            impl From<$ty> for Value {
                fn from(value: $ty) -> Self {
                    (&*value).into()
                }
            }
        )+
    };
}

impl_into_str! {
    &mut str; &EcoString; &mut EcoString; &String; &mut String;
}

impl From<Lambda> for Value {
    #[inline]
    fn from(value: Lambda) -> Self {
        Self::Lambda(value)
    }
}

impl From<Vector<Value>> for Value {
    #[inline]
    fn from(value: Vector<Value>) -> Self {
        Self::List(value)
    }
}

impl From<Vec<Value>> for Value {
    #[inline]
    fn from(value: Vec<Value>) -> Self {
        Vector::from(value).into()
    }
}

impl From<&[Value]> for Value {
    fn from(value: &[Value]) -> Self {
        let mut v = Vector::new();
        v.extend(value.iter().cloned());
        v.into()
    }
}

impl From<&mut [Value]> for Value {
    fn from(value: &mut [Value]) -> Self {
        let mut v = Vector::new();
        v.extend(value.iter().cloned());
        v.into()
    }
}

impl<const N: usize> From<[Value; N]> for Value {
    fn from(value: [Value; N]) -> Self {
        let mut v = Vector::new();
        v.extend(value.into_iter());
        v.into()
    }
}

impl<const N: usize> From<&[Value; N]> for Value {
    #[inline]
    fn from(value: &[Value; N]) -> Self {
        (&value[..]).into()
    }
}

impl<const N: usize> From<&mut [Value; N]> for Value {
    #[inline]
    fn from(value: &mut [Value; N]) -> Self {
        (&*value).into()
    }
}

impl From<Environment> for Value {
    #[inline]
    fn from(value: Environment) -> Self {
        Self::Environment(value)
    }
}

impl<T: Into<Value>> From<Option<T>> for Value {
    fn from(value: Option<T>) -> Self {
        if let Some(value) = value {
            value.into()
        } else {
            Self::Nil
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unspecified => write!(f, "#<unspecified>"),
            Self::Nil => write!(f, "#nil"),
            &Self::Boolean(b) => {
                if b {
                    write!(f, "#t")
                } else {
                    write!(f, "#f")
                }
            }
            Self::Character(c) => fmt::Debug::fmt(c, f),
            Self::Integer(i) => fmt::Debug::fmt(i, f),
            Self::String(s) => fmt::Debug::fmt(s, f),
            Self::Symbol(s) => fmt::Debug::fmt(s, f),
            Self::Lambda(l) => fmt::Debug::fmt(l, f),
            Self::List(l) => print_list_debug(f, l.iter(), "(", ")"),
            Self::Var(v) => fmt::Debug::fmt(v, f),
            Self::Environment(e) => fmt::Debug::fmt(e, f),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unspecified => write!(f, "#<unspecified>"),
            Self::Nil => write!(f, "#nil"),
            &Self::Boolean(b) => {
                if b {
                    write!(f, "#t")
                } else {
                    write!(f, "#f")
                }
            }
            Self::Character(c) => fmt::Display::fmt(c, f),
            Self::Integer(i) => fmt::Display::fmt(i, f),
            Self::String(s) => fmt::Display::fmt(s, f),
            Self::Symbol(s) => fmt::Display::fmt(s, f),
            Self::Lambda(l) => fmt::Debug::fmt(l, f),
            Self::List(l) => print_list_display(f, l.iter(), "(", ")"),
            Self::Var(v) => fmt::Display::fmt(v, f),
            Self::Environment(v) => fmt::Display::fmt(v, f),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Value;
    use rug::Integer;

    #[test]
    fn from() {
        assert_eq!(Value::from(1), Value::Integer(Integer::from(1)));
        assert_eq!(Value::from(1u32), Value::Integer(Integer::from(1)));
        assert_eq!(Value::from(false), Value::Boolean(false));
        assert_eq!(Value::from(true), Value::Boolean(true));
        assert_eq!(Value::from("test"), Value::String("test".into()));
    }

    #[test]
    fn debug() {
        assert_eq!(format!("{:?}", Value::Unspecified), "#<unspecified>");
        assert_eq!(format!("{:?}", Value::Nil), "#nil");
        assert_eq!(format!("{:?}", Value::from(true)), "#t");
        assert_eq!(format!("{:?}", Value::from(1)), "1");
        assert_eq!(format!("{:?}", Value::from(Some(1))), "1");
        assert_eq!(format!("{:?}", Value::from(Option::<bool>::None)), "#nil");
    }
}
