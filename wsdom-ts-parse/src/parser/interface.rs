use winnow::{
    combinator::{delimited, opt, preceded, repeat, separated1},
    PResult, Parser,
};

use super::{
    comment::WithComment,
    generic::GenericsDeclaration,
    member::Member,
    ts_type::TsType,
    util::{token, token_word, word1, Parsable},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interface<'a> {
    pub name: &'a str,
    pub generics: GenericsDeclaration<'a>,
    pub extends: Vec<TsType<'a>>,
    pub members: Vec<WithComment<'a, Member<'a>>>,
}

impl<'a> Parsable<'a> for Interface<'a> {
    fn parse(input: &mut &'a str) -> PResult<Self> {
        preceded(
            token_word("interface"),
            (
                word1,
                opt(GenericsDeclaration::parse),
                opt(preceded(
                    token_word("extends"),
                    separated1(TsType::parse, token(',')),
                )),
                delimited(
                    token('{'),
                    repeat(0.., WithComment::<Member>::parse),
                    token('}'),
                ),
            ),
        )
        .map(|(name, generic, extends, members)| Self {
            name,
            generics: generic.unwrap_or_default(),
            extends: extends.unwrap_or_default(),
            members,
        })
        .parse_next(input)
    }
}
