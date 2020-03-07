#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Delimiter {
    Comma,
    Semicolon,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
