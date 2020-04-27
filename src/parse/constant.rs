#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Delimiter {
    Comma,
    Semicolon,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Keyword {
    Data,
    Comp,
    Type,
    Impl,
    Enum,
    Calc,
    Trans,
    Inv,
    Intake,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Operator {
    Plus,
    PlusEquals,
    Minus,
    MinusEquals,
    Multiply,
    MultiplyEquals,
    Divide,
    DivideEquals,
    PowerOf,
    Modulus,
    Assignment,
    IsEquals,
    Not,
    IsNotEquals,
    Accessor,
    RangeMiddle,
    ArrayContinuation,
    Arrow,
    Into,
    LessThan,
    LessThanOrEquals,
    GreaterThan,
    GreaterThanOrEquals,
    And,
    Or,
    ArrayShiftLeft,
    ArrayShiftRight,
    QuestionMark,
    Of,
    OfClass,
    Escape,
}
#[derive(Clone, Copy, Debug)]
pub enum Number {
    Integer(i64),
    UnsignedInt(u64),
    Float(f64),

    Byte(u8),
    Char(u8),
}
impl std::hash::Hash for Number {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        match self {
            Number::Integer(i) => state.write_i64(*i),
            Number::UnsignedInt(i) => state.write_u64(*i),
            Number::Float(f) => {
                let bytes = f.to_bits().to_be_bytes();
                state.write_i64(i64::from_be_bytes(bytes))
            }
            Number::Byte(i) => state.write_u8(*i),
            Number::Char(i) => state.write_u8(*i),
        }
    }
}
impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        match *self {
            Number::Integer(num) => match *other {
                Number::Integer(other_num) => num == other_num,
                _ => false,
            },
            Number::UnsignedInt(num) => match *other {
                Number::UnsignedInt(other_num) => num == other_num,
                _ => false,
            },
            Number::Byte(num) => match *other {
                Number::Byte(other_num) => num == other_num,
                Number::Char(other_num) => num == other_num,
                _ => false,
            },
            Number::Char(num) => match *other {
                Number::Byte(other_num) => num == other_num,
                Number::Char(other_num) => num == other_num,
                _ => false,
            },
            Number::Float(num) => match *other {
                Number::Float(other_num) => num == other_num,
                _ => false,
            },
        }
    }
}
impl Eq for Number {}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Brace {
    Brace,
    Bracket,
    Angle,
    Square,
    Char(u8),
    Quote(String),
    Comment(String),
}
