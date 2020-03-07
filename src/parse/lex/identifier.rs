use crate::parse::constant::Keyword;
use crate::parse::lex::{Lex, LexParseError};
pub fn is_ident_start(c: u8) -> bool {
    (c>=65&&c<=90)||//upper
    (c>=97&&c<=122) //lower
}
pub fn lex_ident(input: &[u8], index: &mut usize, mut c: u8) -> Result<Option<Lex>, LexParseError> {
    *index += 1;
    if *index >= input.len() {
        return Ok(Some(Lex::Identifier((c as char).to_string(), *index)));
    }
    let mut ident = Vec::new();
    let mut keyword = KeywordCluster::of(c);

    ident.push(c);
    c = input[*index];

    let mut index_from_start = 0usize;
    while is_ident_body(c) {
        ident.push(c);
        if let Some(k) = keyword {
            index_from_start += 1;
            keyword = k.keyword(c, index_from_start);
        }
        *index += 1;
        if *index >= input.len() {
            break;
        }
        c = input[*index];
    }
    if let Some(k) = keyword {
        if k.size_is_valid(index_from_start + 1) {
            return Ok(Some(Lex::Keyword(
                k.as_keyword(index_from_start + 1),
                *index,
            )));
        }
    }
    return Ok(Some(Lex::Identifier(
        ident.into_iter().map(|x| x as char).collect(),
        *index,
    )));
}
fn is_ident_body(c: u8) -> bool {
    (c>=65&&c<=90)||//upper
    (c>=97&&c<=122)||//lower
    (c>=48&&c<=57)||//number
    (c==45)||//hyphen
    (c==95) //underscore
}

#[derive(Debug)]
enum KeywordCluster {
    Data,
    Comp,
    CompCalc,
    Calc,
    Type,
    TypeTrans,
    Trans,
    Impl,
    ImplInvIntake,
    InvIntake,
    Inv,
    Intake,
}
impl KeywordCluster {
    pub fn of(candidate: u8) -> Option<KeywordCluster> {
        match candidate as char {
            'd' => Some(KeywordCluster::Data),
            'i' => Some(KeywordCluster::ImplInvIntake),
            'c' => Some(KeywordCluster::CompCalc),
            't' => Some(KeywordCluster::TypeTrans),
            _ => None,
        }
    }
    pub fn size_is_valid(&self, size: usize) -> bool {
        match *self {
            KeywordCluster::Data => size == 4,
            KeywordCluster::Comp => size == 4,
            KeywordCluster::Type => size == 4,
            KeywordCluster::Impl => size == 4,
            KeywordCluster::Trans => size == 5,
            KeywordCluster::Calc => size == 4,
            KeywordCluster::Inv => size == 3,
            KeywordCluster::Intake => size == 6,
            _ => false,
        }
    }
    pub fn as_keyword(&self, _size: usize) -> Keyword {
        match *self {
            KeywordCluster::Data => Keyword::Data,
            KeywordCluster::Comp => Keyword::Comp,
            KeywordCluster::Type => Keyword::Type,
            KeywordCluster::Impl => Keyword::Impl,
            KeywordCluster::Inv => Keyword::Inv,
            KeywordCluster::Calc => Keyword::Calc,
            KeywordCluster::Trans => Keyword::Trans,
            KeywordCluster::Intake => Keyword::Intake,
            _ => panic!("{:?} should be unreachable", self),
        }
    }
    pub fn keyword(&self, candidate: u8, index: usize) -> Option<KeywordCluster> {
        let c: char = candidate as char;
        match *self {
            KeywordCluster::Data => match index {
                0 => match c {
                    'd' => Some(KeywordCluster::Data),
                    _ => None,
                },
                1 | 3 => match c {
                    'a' => Some(KeywordCluster::Data),
                    _ => None,
                },
                2 => match c {
                    't' => Some(KeywordCluster::Data),
                    _ => None,
                },
                _ => None,
            },
            KeywordCluster::CompCalc => match index {
                0 => match c {
                    'c' => Some(KeywordCluster::CompCalc),
                    _ => None,
                },
                1 => match c {
                    'a' => Some(KeywordCluster::Calc),
                    'o' => Some(KeywordCluster::Comp),
                    _ => None,
                },
                _ => None,
            },
            KeywordCluster::Calc => match index {
                2 => match c {
                    'l' => Some(KeywordCluster::Calc),
                    _ => None,
                },
                3 => match c {
                    'c' => Some(KeywordCluster::Calc),
                    _ => None,
                },
                _ => None,
            },
            KeywordCluster::Comp => match index {
                2 => match c {
                    'm' => Some(KeywordCluster::Comp),
                    _ => None,
                },
                3 => match c {
                    'p' => Some(KeywordCluster::Comp),
                    _ => None,
                },
                _ => None,
            },
            KeywordCluster::TypeTrans => match index {
                0 => match c {
                    't' => Some(KeywordCluster::TypeTrans),
                    _ => None,
                },
                1 => match c {
                    'y' => Some(KeywordCluster::Type),
                    'r' => Some(KeywordCluster::Trans),
                    _ => None,
                },
                _ => None,
            },
            KeywordCluster::Type => match index {
                2 => match c {
                    'p' => Some(KeywordCluster::Type),
                    _ => None,
                },
                3 => match c {
                    'e' => Some(KeywordCluster::Type),
                    _ => None,
                },
                _ => None,
            },
            KeywordCluster::Trans => match index {
                2 => match c {
                    'a' => Some(KeywordCluster::Trans),
                    _ => None,
                },
                3 => match c {
                    'n' => Some(KeywordCluster::Trans),
                    _ => None,
                },
                4 => match c {
                    's' => Some(KeywordCluster::Trans),
                    _ => None,
                },
                _ => None,
            },
            KeywordCluster::ImplInvIntake => match index {
                0 => match c {
                    'i' => Some(KeywordCluster::ImplInvIntake),
                    _ => None,
                },
                1 => match c {
                    'm' => Some(KeywordCluster::Impl),
                    'n' => Some(KeywordCluster::InvIntake),
                    _ => None,
                },
                _ => None,
            },
            KeywordCluster::Impl => match index {
                2 => match c {
                    'p' => Some(KeywordCluster::Impl),
                    _ => None,
                },
                3 => match c {
                    'l' => Some(KeywordCluster::Impl),
                    _ => None,
                },
                _ => None,
            },
            KeywordCluster::InvIntake => match index {
                2 => match c {
                    'v' => Some(KeywordCluster::Inv),
                    't' => Some(KeywordCluster::Intake),
                    _ => None,
                },
                _ => None,
            },
            KeywordCluster::Inv => match index {
                2 => match c {
                    'v' => Some(KeywordCluster::Inv),
                    _ => None,
                },
                _ => None,
            },
            KeywordCluster::Intake => match index {
                3 => match c {
                    'a' => Some(KeywordCluster::Intake),
                    _ => None,
                },
                4 => match c {
                    'k' => Some(KeywordCluster::Intake),
                    _ => None,
                },
                5 => match c {
                    'e' => Some(KeywordCluster::Intake),
                    _ => None,
                },
                _ => None,
            },
        }
    }
}
