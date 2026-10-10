use crate::{grammar, CstText, Item, Rule, Span};

/// items の結果を読みやすい文字列の列にする
fn describe(text: &str, parser: fn(&str) -> Result<crate::Cst, peg::error::ParseError<peg::str::LineCol>>) -> Vec<String> {
    let csttext = CstText::parse(text, parser).expect("parse failed");
    csttext
        .items(&csttext.cst)
        .iter()
        .map(|item| match item {
            Item::Node(c) => format!("{:?} {:?}", c.rule, csttext.get_text(c)),
            Item::Comment { span, own_line } => {
                format!("Comment {:?} own_line={}", csttext.get_text_from_span(*span), own_line)
            }
            Item::BlankLines(n) => format!("BlankLines {}", n),
        })
        .collect()
}

#[test]
fn comment_between_then_and_else() {
    assert_eq!(
        describe("if x then y % c\nelse z", |t| grammar::ctrl_if(t)),
        vec![
            r#"expr "x""#,
            r#"ctrl_then "then y""#,
            r#"Comment "% c" own_line=false"#,
            r#"ctrl_else "else z""#,
        ]
    );
}

#[test]
fn comment_on_own_line() {
    assert_eq!(
        describe("if x then y\n  % c\nelse z", |t| grammar::ctrl_if(t)),
        vec![
            r#"expr "x""#,
            r#"ctrl_then "then y""#,
            r#"Comment "% c" own_line=true"#,
            r#"ctrl_else "else z""#,
        ]
    );
}

#[test]
fn comment_inside_empty_list() {
    // 子がなくても隙間の comment は拾える
    assert_eq!(
        describe("[ % c\n ]", |t| grammar::list(t)),
        vec![r#"Comment "% c" own_line=false"#]
    );
}

#[test]
fn blank_lines_between_statements() {
    assert_eq!(
        describe("let a = 1\n\n\nlet b = 2\nlet c = 3", |t| grammar::preamble(t)),
        vec![
            r#"let_stmt "let a = 1""#,
            "BlankLines 2",
            r#"let_stmt "let b = 2""#,
            r#"let_stmt "let c = 3""#,
        ]
    );
}

#[test]
fn blank_line_after_comment() {
    assert_eq!(
        describe("let a = 1\n% c\n\nlet b = 2", |t| grammar::preamble(t)),
        vec![
            r#"let_stmt "let a = 1""#,
            r#"Comment "% c" own_line=true"#,
            "BlankLines 1",
            r#"let_stmt "let b = 2""#,
        ]
    );
}

#[test]
fn percent_in_tokens_is_not_comment() {
    // 文字列
    let c = CstText::parse("`a % b`", grammar::const_string);
    let c = c.unwrap();
    assert!(c.items(&c.cst).is_empty());
    // 水平モードのエスケープ・文章
    let c = CstText::parse(r"{ a \% b % real\n}".replace(r"\n", "\n").as_str(), |t| grammar::horizontal_text(t)).unwrap();
    let texts: Vec<_> = c
        .items(&c.cst.inner[0])
        .into_iter()
        .filter_map(|i| match i {
            Item::Comment { span, .. } => Some(c.get_text_from_span(span).to_owned()),
            _ => None,
        })
        .collect();
    assert_eq!(texts, vec!["% real".to_owned()]);
}

#[test]
fn comment_span_is_exact() {
    let c = CstText::parse("[1; % c\r\n 2]", grammar::list).unwrap();
    let spans: Vec<Span> = c
        .items(&c.cst)
        .into_iter()
        .filter_map(|i| match i {
            Item::Comment { span, .. } => Some(span),
            _ => None,
        })
        .collect();
    assert_eq!(spans, vec![Span { start: 4, end: 7 }]);
    assert_eq!(c.cst.rule, Rule::list);
}
