use chumsky::{Parser as _, prelude::*};
use freya::prelude::*;

use crate::components::markdown::parser::{Inline, TextSpan};

#[derive(Debug, Clone)]
pub enum InlineParserElement {
    Text(String),
    UserMention(String),
    ChannelMention(String),
    RoleMention(String),
    Emoji(String),
    Link {
        url: String,
        title: Option<String>,
        text: String,
    },
    Hyperlink(String),
    Spoiler(String),
}

pub fn id_parser<'a>() -> impl chumsky::Parser<'a, &'a str, String> {
    one_of("0123456789ABCDEFGHJKMNPQRSTVWXYZ")
        .repeated()
        .exactly(26)
        .collect()
}

pub fn user_mention_parser<'a>() -> impl chumsky::Parser<'a, &'a str, InlineParserElement> {
    just("@")
        .ignore_then(id_parser())
        .delimited_by(just("<"), just(">"))
        .map(InlineParserElement::UserMention)
}

pub fn channel_mention_parser<'a>() -> impl chumsky::Parser<'a, &'a str, InlineParserElement> {
    just("#")
        .ignore_then(id_parser())
        .delimited_by(just("<"), just(">"))
        .map(InlineParserElement::ChannelMention)
}

pub fn role_mention_parser<'a>() -> impl chumsky::Parser<'a, &'a str, InlineParserElement> {
    just("%")
        .ignore_then(id_parser())
        .delimited_by(just("<"), just(">"))
        .map(InlineParserElement::RoleMention)
}

pub fn emoji_mention_parser<'a>() -> impl chumsky::Parser<'a, &'a str, InlineParserElement> {
    id_parser()
        .delimited_by(just(":"), just(":"))
        .map(InlineParserElement::Emoji)
}

pub fn link_parser<'a>() -> impl chumsky::Parser<'a, &'a str, InlineParserElement> {
    none_of(']')
        .repeated()
        .collect()
        .delimited_by(just("["), just("]"))
        .then(
            none_of(" )")
                .repeated()
                .collect::<String>()
                // .then(
                //     just(' ').repeated().at_least(1).ignore_then(
                //         none_of("\"")
                //             .repeated()
                //             .collect::<String>()
                //             .delimited_by(just("\""), just("\""))
                //             .or_not(),
                //     ),
                // )
                .delimited_by(just('('), just(')')),
        )
        .map(|(text, url)| InlineParserElement::Link {
            url,
            title: None,
            text,
        })
}

pub fn spoiler_parser<'a>() -> impl chumsky::Parser<'a, &'a str, InlineParserElement> {
    any()
        .and_is(just("||").not())
        .repeated()
        .collect::<String>()
        .padded_by(just("||"))
        .map(InlineParserElement::Spoiler)
}

pub fn hyperlink_parser<'a>() -> impl chumsky::Parser<'a, &'a str, InlineParserElement> {
    any()
        .filter(|c: &char| !c.is_whitespace())
        .repeated()
        .collect::<String>()
        .filter(|text| Url::parse(text).is_ok_and(|url| ["https", "http"].contains(&url.scheme())))
        .map(InlineParserElement::Hyperlink)
}

pub fn custom_element_parser<'a>() -> impl chumsky::Parser<'a, &'a str, InlineParserElement> {
    choice((
        user_mention_parser(),
        channel_mention_parser(),
        role_mention_parser(),
        emoji_mention_parser(),
        link_parser(),
        hyperlink_parser(),
        spoiler_parser(),
    ))
}

pub fn element_parser<'a>() -> impl chumsky::Parser<'a, &'a str, InlineParserElement> {
    custom_element_parser().or(any::<&str, _>().map(|c| InlineParserElement::Text(c.to_string())))
}

pub fn inline_parser<'a>() -> impl chumsky::Parser<'a, &'a str, Vec<InlineParserElement>> {
    element_parser().repeated().collect()
}

pub fn map_element(
    element: InlineParserElement,
    bold: bool,
    italic: bool,
    strikethrough: bool,
) -> Inline {
    match element {
        InlineParserElement::Text(text) => Inline::Span(TextSpan {
            text,
            bold,
            italic,
            strikethrough,
            code: false,
            link: false,
            spoiler: false,
        }),
        InlineParserElement::UserMention(id) => Inline::UserMention { id },
        InlineParserElement::ChannelMention(id) => Inline::ChannelMention { id },
        InlineParserElement::RoleMention(id) => Inline::RoleMention { id },
        InlineParserElement::Emoji(id) => Inline::Emoji { id },
        InlineParserElement::Link { url, title, text } => Inline::Link {
            url,
            title,
            text: parse_elements(&text, bold, italic, strikethrough),
        },
        InlineParserElement::Hyperlink(text) => Inline::Span(TextSpan {
            text,
            bold,
            italic,
            strikethrough,
            code: false,
            link: true,
            spoiler: false,
        }),
        InlineParserElement::Spoiler(text) => {
            Inline::Spoiler(parse_elements(&text, bold, italic, strikethrough))
        }
    }
}

pub fn parse_elements(content: &str, bold: bool, italic: bool, strikethrough: bool) -> Vec<Inline> {
    // vec![
    //     Inline::Span(TextSpan { text: content.to_string(), bold, italic, strikethrough, code: false, link: false, spoiler: false })
    // ]
    let elements = inline_parser()
        .parse(content)
        .into_output()
        .unwrap_or_default()
        .into_iter()
        .fold(Vec::new(), |mut vec, element| {
            match element {
                InlineParserElement::Text(current) => {
                    if matches!(vec.last(), Some(InlineParserElement::Text(_))) {
                        let Some(InlineParserElement::Text(last)) = vec.pop() else {
                            unreachable!()
                        };

                        vec.push(InlineParserElement::Text(last + &current));
                    } else {
                        vec.push(InlineParserElement::Text(current));
                    }
                }
                element => vec.push(element),
            };

            vec
        });

    elements
        .into_iter()
        .map(|element| map_element(element, bold, italic, strikethrough))
        .collect()
}
