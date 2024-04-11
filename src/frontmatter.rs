use std::{borrow::BorrowMut, collections::{BTreeMap, HashMap}, ops::{Index, IndexMut}};

use serde_yaml::{value::TaggedValue, Mapping, Value};



#[derive(Debug, Clone)]
pub struct Frontmatter {
    text: String,
    data: BTreeMap<String, Value>,
}

impl Frontmatter {
    pub fn new() -> Self { 
        Self {
            text: "".into(),
            data: BTreeMap::new()
        }
    }
}

impl PartialEq for Frontmatter {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text &&
            self.data == other.data
    }
}

pub struct FrontmatterValue {
    value: Value
}

pub trait FrontmatterHelper {
    fn as_string(&self) -> Option<String>;
}

impl FrontmatterHelper for Value {
    fn as_string(&self) -> Option<String> {
        match self {
            Value::String(text) => Some(text.clone()),
            Value::Bool(val)    => Some(val.to_string()),
            Value::Number(val)  => Some(val.to_string()),
            _ => None
        }
    }
}

impl Index<&str> for Frontmatter {
    type Output = Value;

    fn index(&self, key: &str) -> &Self::Output {
        if self.data.contains_key(key) {
            &self.data[key]
        } else {
            &Value::Null
        }
    }
}
impl Index<String> for Frontmatter {
    type Output = Value;

    fn index(&self, key: String) -> &Self::Output {
        if self.data.contains_key(key.as_str()) {
            &self.data[key.as_str()]
        } else {
            &Value::Null
        }
    }
}


fn replace_percentage(val: Value) -> Value {
    match val {
        Value::Null => Value::Null,
        Value::Bool(value) => Value::Bool(value),
        Value::Number(number) => Value::Number(number),
        Value::String(text) => Value::String(text.replace("_%", "%")),
        Value::Sequence(vec) => {
            let mut values : Vec<Value> = vec![];
            for item in vec {
                values.push(replace_percentage(item));
            }
            Value::Sequence(values)
        },
        Value::Mapping(mapping) => {
            let mut result = Mapping::new();
            for (key, value) in mapping {
                result.insert(
                    replace_percentage(key),
                    replace_percentage(value),
                );
            }
            Value::Mapping(result)
        },
        Value::Tagged(box tagged) => Value::Tagged(Box::new(TaggedValue{tag: tagged.tag, value: replace_percentage(tagged.value)})),
    }
}

pub fn parse_frontmatter(text: &str) -> (Option<Frontmatter>, &str) {
    if !(text.starts_with("---\n") || text.starts_with("---\r\n")) {
        return (None, text);
    }

    if let Some(end) = text[3..].find("\n---") {
        let text = &text[3..];
        let remaining_text = &text[(end + 4)..];
        let mut text = text[..end].trim().to_string();
        if text.contains("%") {
            text = text.replace("%", "_%");
        }

        if let Ok(data)  = serde_yaml::from_str::<BTreeMap<String, Value>>(text.as_str()) {

            let mut result = BTreeMap::<String, Value>::new();
            for (item, text) in data {
                result.insert(item, replace_percentage(text));
            }

            text = text.replace("_%", "%");
            (Some(Frontmatter{ 
                text,
                data: result
            }), remaining_text)
        } 
        else if text.len() == 0 {
            (Some(Frontmatter::new()), remaining_text)
        }else {
            (None, remaining_text)
        }
    } else {
        (None, text)
    }
}


#[cfg(test)]
mod tests {
    use crate::frontmatter::*;

    #[test]
    fn basic_parsing() {
        let (fm, remaining) = parse_frontmatter("---\n---");
        assert_eq!(fm, Some(Frontmatter{ text: "".into(), data: BTreeMap::new()}));
        assert_eq!(remaining, "");
    }
    
    #[test]
    fn actual_results_parsing() {
        let text = 
r##"---
this: is an element
another : element
---and this would be the remaining text
"##;
        let (fm, remaining) = parse_frontmatter(text);
        assert_eq!(fm, 
            Some(Frontmatter{ 
                text: "this: is an element\nanother : element".into(), 
                data: BTreeMap::from([
                    ("this".into(), Value::String("is an element".into())),
                    ("another".into(), Value::String("element".into())),
                ])
            }));
        assert_eq!(remaining, "and this would be the remaining text\n");
    }

    #[test]
    fn parsing_with_percentage() {
        let text = 
r##"---
this: %p is a penis
and-a-list: 
    - %p
    - last
another : element
---and this would be the remaining text
"##;

        let (fm, remaining) = parse_frontmatter(text);
        assert_eq!(fm, 
            Some(Frontmatter{ 
                text: "this: %p is a penis\nand-a-list: \n    - %p\n    - last\nanother : element".into(), 
                data: BTreeMap::from([
                    ("this".into(), Value::String("%p is a penis".into())),
                    ("and-a-list".into(), Value::Sequence(vec![
                        Value::String("%p".into()),
                        Value::String("last".into()),
                    ])),
                    ("another".into(), Value::String("element".into())),
                ])
            }));
        assert_eq!(remaining, "and this would be the remaining text\n");
    }
}

