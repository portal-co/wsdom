use winnow::{
    combinator::{alt, delimited, opt, preceded, separated_pair, terminated},
    token::one_of,
    PResult, Parser,
};

use super::{
    field::Field,
    method::Method,
    ts_type::TsType,
    util::{token, token_word, word1, Parsable},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Member<'a> {
    Method(Method<'a>),
    Field(Field<'a>),
    Getter(Getter<'a>),
    Setter(Setter<'a>),
}
impl<'a> Parsable<'a> for Member<'a> {
    fn parse(input: &mut &'a str) -> PResult<Self> {
        terminated(
            alt((
                Method::parse.map(Self::Method),
                Field::parse.map(Self::Field),
                Getter::parse.map(Self::Getter),
                Setter::parse.map(Self::Setter),
            )),
            opt(token(one_of((',', ';')))),
        )
        .parse_next(input)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Getter<'a> {
    pub name: &'a str,
    pub ret: TsType<'a>,
}

impl<'a> Parsable<'a> for Getter<'a> {
    fn parse(input: &mut &'a str) -> PResult<Self> {
        preceded(
            token_word("get"),
            separated_pair(word1, (token('('), token(')'), token(':')), TsType::parse),
        )
        .map(|(name, ret)| Self { name, ret })
        .parse_next(input)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Setter<'a> {
    pub name: &'a str,
    pub arg_name: &'a str,
    pub arg_ty: TsType<'a>,
}

impl<'a> Parsable<'a> for Setter<'a> {
    fn parse(input: &mut &'a str) -> PResult<Self> {
        preceded(
            token_word("set"),
            (
                word1,
                delimited(
                    token('('),
                    separated_pair(word1, token(':'), TsType::parse),
                    token(')'),
                ),
            ),
        )
        .map(|(name, (arg_name, arg_ty))| Self {
            name,
            arg_name,
            arg_ty,
        })
        .parse_next(input)
    }
}
