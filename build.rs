use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use convert_case::ccase;
use fluent_syntax::ast::{Entry, Expression, InlineExpression, PatternElement};

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
struct LocEntry {
    key: String,
    value: String,
    path: String,
}

impl PartialOrd for LocEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl Ord for LocEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key.cmp(&other.key)
    }
}

fn handle_inline_expression(expr: InlineExpression<String>, terms_to_values: &HashMap<String, String>) -> String {
    match expr {
        InlineExpression::StringLiteral { value } => value,
        InlineExpression::NumberLiteral { value } => value,
        InlineExpression::FunctionReference { .. } => String::from("unhandled 1"),
        InlineExpression::MessageReference { .. } => String::from("unhandled 2"),
        InlineExpression::TermReference { id, .. } => {
            let Some(term_value) = terms_to_values.get(&id.name) else {
                return String::from("unhandled 5");
            };

            term_value.to_owned()
        },
        InlineExpression::VariableReference { id } => format!("{{{}}}", id.name),
        InlineExpression::Placeable { .. } => String::from("unhandled 4"),
    }
}

fn main() {
    println!("cargo:rerun-if-changed=locales");

    // ZJ-TODO: use a hashmap, such that we can confirm all locales have all keys
    let mut loc_entries = HashSet::new();
    for maybe_locale_dir in walkdir::WalkDir::new("locales") {
        let Ok(locale_dir) = maybe_locale_dir else {
            return;
        };

        if !locale_dir.file_type().is_dir() {
            continue;
        }

        for maybe_entry in walkdir::WalkDir::new(locale_dir.path()) {
            let Ok(entry) = maybe_entry else {
                return;
            };

            if !entry.file_type().is_file() {
                continue;
            }

            let contents = std::fs::read_to_string(entry.path()).unwrap();
            let ast = fluent_syntax::parser::parse(contents).unwrap();

            let terms_to_values = ast.body.iter()
                .filter(|entry| matches!(entry, Entry::Term(..)))
                .map(|entry| {
                    let Entry::Term(term) = entry else {
                        panic!("what");
                    };

                    let term_id = term.id.name.clone();
                    let Some(PatternElement::TextElement { value }) = term.value.elements.first() else {
                        panic!("what 2");
                    };

                    return (term_id, value.to_owned());
                })
                .collect::<HashMap<_, _>>();

            ast.body.iter().for_each(|statement| {
                match statement {
                    Entry::Message(m) => {
                        loc_entries.insert(LocEntry {
                            key: m.id.name.clone(),
                            value: m.value.as_ref().unwrap().elements.clone().into_iter().map(|e| match e {
                                PatternElement::TextElement { value } => value,
                                PatternElement::Placeable { expression } => {
                                    match expression {
                                        Expression::Inline(inline) => handle_inline_expression(inline, &terms_to_values),
                                        Expression::Select { selector, variants } => {
                                            let selector_text = handle_inline_expression(selector, &terms_to_values);
                                            let mut variants_str = String::new();
                                            for variant in variants {
                                                for element in &variant.value.elements {
                                                    let PatternElement::TextElement { value } = element else {
                                                        continue;
                                                    };
                                                    variants_str += &format!("'{}' / ", value);
                                                }
                                            }
                                            // Remove the trailing slash and extra space
                                            variants_str.truncate(variants_str.len() - 2);
                                            format!("{selector_text} -> {variants_str}")
                                        }
                                    }.to_owned()
                                }
                            })
                                .map(|str| str.replace("\n", " "))
                                .reduce(|acc, next| acc + &next)
                                .unwrap(),
                            path: entry.path().to_string_lossy().into_owned(),
                        });
                    }
                    _ => {}
                }
            })
        }
    }

    let mut rust_enum = "use strum::EnumMessage;\n#[derive(strum::EnumMessage)]\npub enum MessageId {\n".to_string();

    let mut loc_entries_vec = loc_entries
        .into_iter()
        .collect::<Vec<_>>();
    loc_entries_vec.sort();
    for entry in loc_entries_vec {
        rust_enum.push_str(&format!("\t/// > {} \n", entry.value));
        rust_enum.push_str("\t///\n");
        rust_enum.push_str(&format!("\t/// {} \n", entry.path));
        rust_enum.push_str(&format!("\t#[strum(message=\"{}\")]\n", entry.key));
        rust_enum.push_str(&format!("\t{},\n\n", ccase!(pascal, &entry.key)));
    }

    rust_enum.truncate(rust_enum.len() - 1);
    rust_enum += "}";

    rust_enum += r#"

impl MessageId {
    pub fn get(&self) -> &'static str {
        self.get_message().unwrap_or_default()
    }
}
    "#;

    std::fs::write(
        "src/l10n/message_id.rs",
        rust_enum,
    ).unwrap();
}