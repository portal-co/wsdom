use winnow::{
    combinator::{delimited, opt, preceded, separated_pair},
    Parser,
};

use super::{
    generic::GenericsDeclaration,
    ts_type::TsType,
    util::{token, token_word, word1, Parsable},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeAlias<'a> {
    pub name: &'a str,
    pub generics: GenericsDeclaration<'a>,
    pub ty: TsType<'a>,
}

impl<'a> Parsable<'a> for TypeAlias<'a> {
    fn parse(input: &mut &'a str) -> winnow::PResult<Self> {
        delimited(
            preceded(opt(token_word("declare")), token_word("type")),
            separated_pair(
                (word1, opt(GenericsDeclaration::parse)),
                token('='),
                TsType::parse,
            ),
            token(';'),
        )
        .map(|((name, generics), ty)| Self {
            name,
            generics: generics.unwrap_or_default(),
            ty,
        })
        .parse_next(input)
    }
}
