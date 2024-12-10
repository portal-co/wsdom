use winnow::{
    ascii::line_ending,
    combinator::{delimited, repeat},
    PResult, Parser,
};

use crate::parser::util::{quote_backslash_escape, token, token_word};

use self::util::Parsable;

pub mod comment;
pub mod declare_class;
pub mod declare_function;
pub mod declare_var;
pub mod expr;
pub mod field;
pub mod generic;
pub mod interface;
pub mod item;
pub mod member;
pub mod method;
pub mod namespace;
pub mod ts_type;
pub mod type_alias;
pub mod util;

pub fn parse_all<'a>(
    input: &mut &'a str,
) -> PResult<Vec<comment::WithComment<'a, item::Item<'a>>>> {
    loop {
        if line_ending::<_, winnow::error::ContextError>
            .parse_next(input)
            .is_err()
        {
            break;
        }
    }
    loop {
        if comment::Comment::parse.parse_next(input).is_err() {
            break;
        }
    }
    let mut out = Vec::new();
    while !input.is_empty() {
        out.push(comment::WithComment::<item::Item>::parse.parse_next(input)?);
    }
    Ok(out)
}

pub fn parse_imports<'a>(input: &mut &'a str) -> PResult<Vec<&'a str>> {
    fn parse_import<'a>(input: &mut &'a str) -> PResult<&'a str> {
        delimited(
            (
                token("///"),
                token("<"),
                token_word("reference"),
                token_word("lib"),
                token('='),
            ),
            quote_backslash_escape('"'),
            token("/>"),
        )
        .parse_next(input)
    }
    repeat(0.., parse_import).parse_next(input)
}

mod tests {

    #[test]
    fn test1() {
        use super::{
            comment::{Comment, WithComment},
            declare_var::DeclareVar,
            field::{Field, FieldName},
            item::Item,
            member::Member,
            method::{Method, MethodName},
            ts_type::{NamedType, TsType},
            util::Parsable,
        };
        use winnow::Parser;
        let parsed: WithComment<Item> = Parsable::parse
            .parse(
                &mut r#"
// Hello this is a test
declare var Element: {
    prototype: Element;
    new(): Element;
};
            "#,
            )
            .unwrap();
        assert_eq!(
            parsed,
            WithComment {
                comment: vec!(Comment {
                    source: "// Hello this is a test\n"
                }),
                data: Item::DeclareVar(DeclareVar {
                    name: "Element",
                    ty: TsType::Interface {
                        members: vec![
                            WithComment {
                                comment: vec![],
                                data: Member::Field(Field {
                                    readonly: false,
                                    name: FieldName::Name("prototype"),
                                    optional: false,
                                    ty: TsType::Named {
                                        ty: NamedType {
                                            generic: Default::default(),
                                            name: "Element"
                                        }
                                    }
                                })
                            },
                            WithComment {
                                comment: vec![],
                                data: Member::Method(Method {
                                    name: MethodName::Constructor,
                                    generics: Default::default(),
                                    args: vec![],
                                    ret: Some(TsType::Named {
                                        ty: NamedType {
                                            name: "Element",
                                            generic: Default::default()
                                        }
                                    }),
                                    optional: false,
                                    is_static: false
                                })
                            }
                        ]
                    }
                })
            }
        )
    }
    #[test]
    fn test2() {
        use super::{comment::WithComment, item::Item, Parsable};
        use winnow::{combinator::repeat, Parser};

        let mut input = r#"
/**
 * Hahaha
 */
declare var Element: {
    prototype: Element;
    new(): Element;
};

/** [MDN Reference](https://developer.mozilla.org/docs/Web/API/SubmitEvent) */
interface SubmitEvent extends Event {
    /**
     * Returns the element representing the submit button that triggered the form submission, or null if the submission was not triggered by a button.
     *
     * [MDN Reference](https://developer.mozilla.org/docs/Web/API/SubmitEvent/submitter)
     */
    readonly submitter: HTMLElement | null;
}

interface UnderlyingByteSource {
    autoAllocateChunkSize?: number;
    cancel?: UnderlyingSourceCancelCallback;
    pull?: (controller: ReadableByteStreamController) => void | PromiseLike<void>;
    start?: (controller: ReadableByteStreamController) => any;
    type: "bytes";
}
interface ReadableStream<R = any> {
    /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/ReadableStream/locked) */
    readonly locked: boolean;
    /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/ReadableStream/cancel) */
    cancel(reason?: any): Promise<void>;
    /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/ReadableStream/getReader) */
    getReader(options: { mode: "byob" }): ReadableStreamBYOBReader;
    getReader(): ReadableStreamDefaultReader<R>;
    getReader(options?: ReadableStreamGetReaderOptions): ReadableStreamReader<R>;
    /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/ReadableStream/pipeThrough) */
    pipeThrough<T>(transform: ReadableWritablePair<T, R>, options?: StreamPipeOptions): ReadableStream<T>;
    /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/ReadableStream/pipeTo) */
    pipeTo(destination: WritableStream<R>, options?: StreamPipeOptions): Promise<void>;
    /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/ReadableStream/tee) */
    tee(): [ReadableStream<R>, ReadableStream<R>];
}
            "#;

        let parse_res: Result<Vec<_>, _> =
            repeat(0.., WithComment::<Item>::parse).parse_next(&mut input);
        println!("{}", input);
        parse_res.unwrap();
    }
}
