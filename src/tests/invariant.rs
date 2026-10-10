//! 構文を一通り含む入力をパースして、全ノードで不変条件を確認する。
use super::rule_allows_surrounding_space;
use crate::{grammar, Cst, CstText};

fn check(path: &str, text: &str, cst: &Cst, errors: &mut Vec<String>) {
    let t = &text[cst.span.start..cst.span.end];
    if !rule_allows_surrounding_space(cst.rule)
        && (t.starts_with(char::is_whitespace) || t.ends_with(char::is_whitespace))
    {
        errors.push(format!("{path}: {:?} {:?}", cst.rule, t));
    }
    // 子は親の範囲に収まり、順序通りで重ならない
    let mut prev_end = cst.span.start;
    for c in &cst.inner {
        if c.span.start < prev_end || c.span.end > cst.span.end {
            errors.push(format!(
                "{path}: child {:?} {:?} is out of order/range in {:?}",
                c.rule, c.span, cst.rule
            ));
        }
        prev_end = c.span.end;
        check(path, text, c, errors);
    }
}

/// `let-rec ... and`、`sig` / `struct`、型定義など、span が崩れやすい構文を一通り含む入力
const SAMPLE: &str = r#"@require: stdjareport

let-rec f x = g x
and g y = y
let-block +p it = '<>
let-inline \m x = {}
type 'a t = A | B of int constraint 'a :: (| x : int |)
and u = int
module M : sig
  type t
  val f : int -> int constraint 'a :: (| |)
end = struct
  let f x = x
end
in
let-rec h x = 1 | y = 2 in
let a = fun x -> Some x in
document (| title = {a} |) '<>
"#;

#[test]
fn sample_keeps_span_invariants() {
    let mut errors = vec![];
    match CstText::parse(SAMPLE, grammar::program) {
        Ok(c) => check("sample", &c.text, &c.cst, &mut errors),
        Err((lc, _)) => errors.push(format!("sample: parse error at {}:{}", lc.line, lc.column)),
    }
    errors.sort();
    errors.dedup();
    assert!(errors.is_empty(), "{}", errors.join("\n"));
}
