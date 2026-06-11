//! Stage 1 (styles): a deliberately small indentation-based sass reader,
//! sufficient for the hoff-research-card corpus. It resolves `$variables`,
//! expands `+mixin` includes (`=mixin` definitions), composes nested
//! selectors (`&.mod`, `&:after`, descendants, `@media`, `@keyframes`) and
//! returns flat rules with file lines. It is NOT a css engine: whatever the
//! resolver does not recognize ends up on the droplist with file:line.

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Decl {
    pub prop: String,
    pub value: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub selector: String,
    pub decls: Vec<Decl>,
    pub line: usize,
}

#[derive(Default)]
struct Defs {
    vars: HashMap<String, String>,
    mixins: HashMap<String, Vec<Decl>>,
}

/// Parse `module` with `$vars`/`=mixins` taken from both `vars_src` and the
/// module itself. Returns rules in source order.
pub fn parse_sass(module: &str, vars_src: &str) -> Vec<Rule> {
    let mut defs = Defs::default();
    collect_defs(vars_src, &mut defs);
    collect_defs(module, &mut defs);

    let mut rules: Vec<Rule> = Vec::new();
    // Stack of (indent, composed selector, selector line).
    let mut stack: Vec<(usize, String, usize)> = Vec::new();
    let mut lines = logical_lines(module).into_iter().peekable();

    while let Some((indent, text, lineno)) = lines.next() {
        if text.starts_with('$') || text.starts_with("@import") {
            continue;
        }
        if text.starts_with('=') {
            // Skip the mixin body (already collected by collect_defs).
            while let Some((i, _, _)) = lines.peek() {
                if *i > indent {
                    lines.next();
                } else {
                    break;
                }
            }
            continue;
        }
        while let Some((top, _, _)) = stack.last() {
            if *top >= indent {
                stack.pop();
            } else {
                break;
            }
        }
        let (parent, sel_line) = stack
            .last()
            .map(|(_, s, l)| (s.clone(), *l))
            .unwrap_or((String::new(), lineno));
        if let Some((prop, value)) = split_decl(&text) {
            let value = subst_vars(&value, &defs.vars);
            push_decl(
                &mut rules,
                &parent,
                sel_line,
                Decl {
                    prop,
                    value,
                    line: lineno,
                },
            );
        } else if let Some(mixin) = text.strip_prefix('+') {
            if let Some(body) = defs.mixins.get(mixin.trim()) {
                for d in body {
                    let mut d = d.clone();
                    d.line = lineno; // The include site is the honest source line.
                    d.value = subst_vars(&d.value, &defs.vars);
                    push_decl(&mut rules, &parent, sel_line, d);
                }
            }
        } else {
            let sel = compose(&parent, &text);
            stack.push((indent, sel, lineno));
        }
    }
    rules
}

fn collect_defs(src: &str, defs: &mut Defs) {
    let mut lines = logical_lines(src).into_iter().peekable();
    while let Some((indent, text, lineno)) = lines.next() {
        if let Some(rest) = text.strip_prefix('$') {
            if let Some((name, value)) = rest.split_once(':') {
                defs.vars
                    .insert(name.trim().to_string(), value.trim().to_string());
            }
        } else if let Some(name) = text.strip_prefix('=') {
            let mut body = Vec::new();
            while let Some((i, _, _)) = lines.peek() {
                if *i <= indent {
                    break;
                }
                let (_, t, l) = lines.next().unwrap();
                if let Some((prop, value)) = split_decl(&t) {
                    body.push(Decl {
                        prop,
                        value,
                        line: l,
                    });
                }
            }
            defs.mixins.insert(name.trim().to_string(), body);
        } else {
            let _ = lineno;
        }
    }
}

/// Non-empty, non-comment lines as (indent, text, 1-based line). Selector
/// lines ending with a comma are joined with their continuation.
fn logical_lines(src: &str) -> Vec<(usize, String, usize)> {
    let mut out: Vec<(usize, String, usize)> = Vec::new();
    for (idx, raw) in src.lines().enumerate() {
        let text = raw.trim();
        if text.is_empty() || text.starts_with("//") {
            continue;
        }
        let indent = raw.len() - raw.trim_start().len();
        if let Some((_, prev, _)) = out.last_mut()
            && prev.ends_with(',')
        {
            prev.push(' ');
            prev.push_str(text);
            continue;
        }
        out.push((indent, text.to_string(), idx + 1));
    }
    out
}

/// A declaration is `prop: value` where prop is a plain (possibly `-webkit-`)
/// identifier; selectors with pseudo colons (`&:after`) do not qualify.
fn split_decl(text: &str) -> Option<(String, String)> {
    let (head, tail) = text.split_once(':')?;
    let head = head.trim();
    let tail = tail.trim();
    if head.is_empty() || tail.is_empty() {
        return None;
    }
    let ok = head
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if ok && !head.starts_with('@') {
        Some((head.to_string(), tail.to_string()))
    } else {
        None
    }
}

fn subst_vars(value: &str, vars: &HashMap<String, String>) -> String {
    if !value.contains('$') {
        return value.to_string();
    }
    // Longest names first so `$a-b` is not clobbered by `$a`.
    let mut names: Vec<&String> = vars.keys().collect();
    names.sort_by_key(|n| std::cmp::Reverse(n.len()));
    let mut out = value.to_string();
    for name in names {
        let pat = format!("${name}");
        if out.contains(&pat) {
            out = out.replace(&pat, &vars[name]);
        }
    }
    out
}

/// Compose a nested selector line against its parent. Comma groups are kept
/// joined here; `push_decl` fans the rule out per part.
fn compose(parent: &str, sel: &str) -> String {
    let parts: Vec<String> = sel
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|part| {
            if let Some(rest) = part.strip_prefix('&') {
                format!("{parent}{rest}")
            } else if part.starts_with('@') {
                if parent.is_empty() {
                    part.to_string()
                } else {
                    format!("{part} <{parent}>")
                }
            } else if parent.is_empty() {
                part.to_string()
            } else {
                format!("{parent} {part}")
            }
        })
        .collect();
    parts.join(", ")
}

/// Append `decl` to the rule for each comma part of `selector`.
fn push_decl(rules: &mut Vec<Rule>, selector: &str, lineno: usize, decl: Decl) {
    for part in selector.split(", ") {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some(rule) = rules.iter_mut().find(|r| r.selector == part) {
            rule.decls.push(decl.clone());
        } else {
            rules.push(Rule {
                selector: part.to_string(),
                decls: vec![decl.clone()],
                line: lineno,
            });
        }
    }
}

/// Direct declarations of `.class` (exact selector match).
pub fn class_decls<'a>(rules: &'a [Rule], class: &str) -> Option<&'a Rule> {
    let sel = format!(".{class}");
    rules.iter().find(|r| r.selector == sel)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_pseudo_and_mixin() {
        let vars = "=t\n    font-size: 20px\n$c: rgba(1, 2, 3, .5)\n";
        let src = ".a\n    +t\n    color: $c\n    &:after\n        border: 1px solid $c\n";
        let rules = parse_sass(src, vars);
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].selector, ".a");
        assert_eq!(rules[0].decls[0].prop, "font-size");
        assert_eq!(rules[0].decls[1].value, "rgba(1, 2, 3, .5)");
        assert_eq!(rules[1].selector, ".a:after");
    }

    #[test]
    fn comma_group_fans_out() {
        let src = ".b\n    &:before,\n    &:after\n        inset: 0\n";
        let rules = parse_sass(src, "");
        let sels: Vec<&str> = rules.iter().map(|r| r.selector.as_str()).collect();
        assert_eq!(sels, vec![".b:before", ".b:after"]);
        assert_eq!(rules[0].decls[0].prop, "inset");
    }

    #[test]
    fn media_keeps_scope_marker() {
        let src = ".c\n    width: 368px\n    @media only screen and (max-width: \"767px\")\n        width: 100%\n";
        let rules = parse_sass(src, "");
        assert!(
            rules
                .iter()
                .any(|r| r.selector.starts_with("@media") && r.selector.contains("<.c>"))
        );
    }
}
