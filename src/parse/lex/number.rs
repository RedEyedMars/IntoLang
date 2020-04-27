use crate::parse::lex::{Lex, LexParseError};

pub fn is_number_start(c: u8) -> bool {
    (c >= 48 && c <= 57) //number
}

pub fn lex_num(input: &[u8], index: &mut usize, mut c: u8) -> Result<Option<Lex>, LexParseError> {
    *index += 1;
    if *index >= input.len() {
        return Ok(Some(Lex::Integer((c as char).to_string(), *index)));
    }
    let mut num = Vec::new();
    let mut classification = NumberClassification::Integer(NumberStopCondition::CanContinue);
    num.push(c);
    c = input[*index];
    classification = classification.classifiy_num(c);
    loop {
        match classification.condition() {
            NumberStopCondition::NotNumber => break,
            NumberStopCondition::MustEnd => {
                num.push(c);
                *index += 1;
                break;
            }
            NumberStopCondition::CanContinue => {
                num.push(c);
            }
        }
        *index += 1;
        if *index >= input.len() {
            break;
        }
        c = input[*index];
        classification = classification.classifiy_num(c);
    }
    return match classification {
        NumberClassification::Integer(_) => Ok(Some(Lex::Integer(
            num.into_iter().map(|x| x as char).collect(),
            *index,
        ))),
        NumberClassification::Floating(_) => Ok(Some(Lex::Float(
            num.into_iter().map(|x| x as char).collect(),
            *index,
        ))),
    };
}
#[derive(Clone, Copy, Debug)]
enum NumberStopCondition {
    MustEnd,
    CanContinue,
    NotNumber,
}
#[derive(Clone, Copy, Debug)]
enum NumberClassification {
    Integer(NumberStopCondition),
    Floating(NumberStopCondition),
}
impl NumberClassification {
    fn classifiy_num(&self, c: u8) -> NumberClassification {
        match *self {
            NumberClassification::Integer(_) => match is_number_start(c) {
                true => NumberClassification::Integer(NumberStopCondition::CanContinue),
                false => match c {
                    46 => NumberClassification::Floating(NumberStopCondition::CanContinue),
                    102 => NumberClassification::Floating(NumberStopCondition::MustEnd),
                    _ => NumberClassification::Integer(NumberStopCondition::NotNumber),
                },
            },
            NumberClassification::Floating(_) => match is_number_start(c) {
                true => NumberClassification::Floating(NumberStopCondition::CanContinue),
                false => match c {
                    102 => NumberClassification::Floating(NumberStopCondition::MustEnd),
                    _ => NumberClassification::Floating(NumberStopCondition::NotNumber),
                },
            },
        }
    }
    fn condition(&self) -> NumberStopCondition {
        match *self {
            NumberClassification::Integer(condition) => condition,
            NumberClassification::Floating(condition) => condition,
        }
    }
}
