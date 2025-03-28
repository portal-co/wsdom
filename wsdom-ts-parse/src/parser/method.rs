use winnow::{
    combinator::{alt, delimited, opt, preceded, separated0, separated_pair},
    PResult, Parser,
};

use super::{
    generic::GenericsDeclaration,
    ts_type::TsType,
    util::{token, token_word, word0, word1, Parsable},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Method<'a> {
    pub name: MethodName<'a>,
    pub generics: GenericsDeclaration<'a>,
    pub args: Vec<MethodArg<'a>>,
    pub ret: Option<TsType<'a>>,
    pub optional: bool,
    pub is_static: bool,
}
impl<'a> Parsable<'a> for Method<'a> {
    fn parse(input: &mut &'a str) -> PResult<Self> {
        (
            (
                opt(token_word("static")),
                MethodName::parse,
                opt(token("?")),
                opt(GenericsDeclaration::parse),
                delimited(
                    token('('),
                    separated0(MethodArg::parse, token(',')),
                    token(')'),
                ),
            ),
            opt(preceded(token(':'), TsType::parse)),
        )
            .parse_next(input)
            .map(|((is_static, name, optional, generics, args), ret)| Self {
                name,
                generics: generics.unwrap_or_default(),
                args,
                ret,
                optional: optional.is_some(),
                is_static: is_static.is_some(),
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodArg<'a> {
    pub name: &'a str,
    pub optional: bool,
    pub ty: TsType<'a>,
    pub variadic: bool,
}
impl<'a> Parsable<'a> for MethodArg<'a> {
    fn parse(input: &mut &'a str) -> PResult<Self> {
        separated_pair(
            (opt(token("...")), word1, opt(token('?'))),
            token(':'),
            TsType::parse,
        )
        .map(|((variadic, name, optional), ty)| Self {
            name,
            optional: optional.is_some(),
            ty,
            variadic: variadic.is_some(),
        })
        .parse_next(input)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MethodName<'a> {
    Nothing,
    Constructor,
    Iterator,
    Name(&'a str),
}
impl<'a> Parsable<'a> for MethodName<'a> {
    fn parse(input: &mut &'a str) -> PResult<Self> {
        alt((
            delimited(token('['), "Symbol.iterator", token(']')).map(|_| Self::Iterator),
            word0.map(|s| match s {
                "" => Self::Nothing,
                "new" | "constructor" => Self::Constructor,
                s => Self::Name(s),
            }),
        ))
        .parse_next(input)
    }
}
