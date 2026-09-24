//! A JSON Schema validator for the keywords `uir::schema()` uses, and no
//! more: `$ref` into `$defs`, `type`, `const`, `enum`, `anyOf`,
//! `properties`, `required`, `additionalProperties`, `propertyNames`,
//! `items`, `minLength`, `minimum` and `maximum`. A keyword it does not
//! know fails the test that meets it, so the schema can never lean on a
//! rule this check skips. It lives in the tests so the crate takes no
//! schema dependency.

use serde_json::Value;

/// Annotations: they say something to a reader and check nothing.
const ANNOTATIONS: &[&str] = &["$schema", "$defs", "title", "description"];

/// Every place `doc` breaks `schema`, as `pointer: reason`.
pub fn validate(schema: &Value, doc: &Value) -> Vec<String> {
    let mut out = Vec::new();
    check(schema, schema, doc, "", &mut out);
    out
}

fn check(root: &Value, s: &Value, v: &Value, at: &str, out: &mut Vec<String>) {
    let Some(rules) = s.as_object() else {
        panic!("a schema is an object, not {s}");
    };
    for (key, rule) in rules {
        match key.as_str() {
            k if ANNOTATIONS.contains(&k) => {}
            "$ref" => {
                let path = rule
                    .as_str()
                    .and_then(|r| r.strip_prefix('#'))
                    .expect("a local ref");
                let target = root
                    .pointer(path)
                    .unwrap_or_else(|| panic!("no {path} in the schema"));
                check(root, target, v, at, out);
            }
            "type" => {
                let fits = match rule.as_str().expect("one type") {
                    "object" => v.is_object(),
                    "array" => v.is_array(),
                    "string" => v.is_string(),
                    "integer" => v.is_i64() || v.is_u64(),
                    "number" => v.is_number(),
                    "boolean" => v.is_boolean(),
                    "null" => v.is_null(),
                    other => panic!("no type {other}"),
                };
                if !fits {
                    out.push(format!("{at}: {v} is not of type {rule}"));
                }
            }
            "const" => {
                if v != rule {
                    out.push(format!("{at}: {v} is not {rule}"));
                }
            }
            "enum" => {
                let options = rule.as_array().expect("a list");
                if !options.contains(v) {
                    out.push(format!("{at}: {v} is none of {rule}"));
                }
            }
            "anyOf" => {
                let branches = rule.as_array().expect("a list");
                let fits = branches.iter().any(|b| {
                    let mut errors = Vec::new();
                    check(root, b, v, at, &mut errors);
                    errors.is_empty()
                });
                if !fits {
                    out.push(format!("{at}: {v} fits none of its forms"));
                }
            }
            "properties" => {
                let (Some(props), Some(object)) = (rule.as_object(), v.as_object()) else {
                    continue;
                };
                for (name, sub) in props {
                    if let Some(field) = object.get(name) {
                        check(root, sub, field, &format!("{at}/{name}"), out);
                    }
                }
            }
            "required" => {
                let Some(object) = v.as_object() else {
                    continue;
                };
                for name in rule.as_array().expect("a list") {
                    let name = name.as_str().expect("a name");
                    if !object.contains_key(name) {
                        out.push(format!("{at}: missing {name}"));
                    }
                }
            }
            "additionalProperties" => {
                let Some(object) = v.as_object() else {
                    continue;
                };
                let known = rules.get("properties").and_then(Value::as_object);
                for (name, field) in object {
                    if known.is_some_and(|k| k.contains_key(name)) {
                        continue;
                    }
                    match rule {
                        Value::Bool(false) => out.push(format!("{at}: no property {name}")),
                        Value::Bool(true) => {}
                        sub => check(root, sub, field, &format!("{at}/{name}"), out),
                    }
                }
            }
            "propertyNames" => {
                let Some(object) = v.as_object() else {
                    continue;
                };
                for name in object.keys() {
                    check(
                        root,
                        rule,
                        &Value::String(name.clone()),
                        &format!("{at}/{name}"),
                        out,
                    );
                }
            }
            "items" => {
                let Some(list) = v.as_array() else {
                    continue;
                };
                for (i, item) in list.iter().enumerate() {
                    check(root, rule, item, &format!("{at}/{i}"), out);
                }
            }
            "minLength" => {
                let min = rule.as_u64().expect("a length");
                if v.as_str().is_some_and(|t| (t.chars().count() as u64) < min) {
                    out.push(format!("{at}: shorter than {min}"));
                }
            }
            "minimum" => {
                if let (Some(n), Some(min)) = (v.as_f64(), rule.as_f64()) {
                    if n < min {
                        out.push(format!("{at}: {n} is below {min}"));
                    }
                }
            }
            "maximum" => {
                if let (Some(n), Some(max)) = (v.as_f64(), rule.as_f64()) {
                    if n > max {
                        out.push(format!("{at}: {n} is above {max}"));
                    }
                }
            }
            other => panic!("the test validator does not know the keyword {other}"),
        }
    }
}
