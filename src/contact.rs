
use anyhow::{anyhow, Context, Result};

use crate::structured_base_parser::{peek_next_token, eat_token, parse_value};

#[derive(Debug, Default, PartialEq, Clone)]
pub struct ContactDefinition {
    pub id: String,
    pub name: String,
    pub phone: Vec<String>,
    pub email: Vec<String>,
    pub address: Vec<String>,
    pub website: Vec<String>
}

pub fn parse_contact(content: String) -> Result<ContactDefinition> {
    let current = content.lines().nth(0).context("expected a line where there was none")?;

    if current.starts_with("?") && current.chars().nth(1).is_some_and(|c| return char::is_alphabetic(c) || c == '-') {
        let mut buf = content.clone();
        let id = if let Some(token) = peek_next_token(&buf) { token } else { return Err(anyhow!("missing name")); };
        buf = eat_token(&buf, &id);
        let mut contact = ContactDefinition::default();
        contact.id = id[1..].to_string();

        if let Some(token) = peek_next_token(&buf) && token == "{" {
            buf = eat_token(&buf, "{");

            loop {
                let opt_token = peek_next_token(&buf);
                if opt_token.is_none() { break }

                let token = opt_token.expect("token was None somehow");
                if token == "}" { break }

                let ident = token.clone();

                buf = eat_token(&buf, &token);
                let opt_token = peek_next_token(&buf);
                if opt_token.is_none() { break }

                let token = opt_token.expect("token was None somehow");
                if token == "}" { break }
                if token != ":" { break }

                buf = eat_token(&buf, &token);

                match ident.to_lowercase().as_str() {
                    "name" | "legal-name" => {
                        let name;
                        (buf, name) = parse_value(&buf);
                        if !name.is_empty() {
                            contact.name = name[0].clone();
                        }
                    },
                    "phone" | "phonenumber" | "tlf" | "phonenumbers" => {
                        let mut phone_numbers;
                        (buf, phone_numbers) = parse_value(&buf);
                        contact.phone.append(&mut phone_numbers);
                    },
                    "email" | "emails" => {
                        let mut emails;
                        (buf, emails) = parse_value(&buf);
                        contact.email.append(&mut emails);
                    },
                    "address" | "addresses" => {
                        let mut adresses;
                        (buf, adresses) = parse_value(&buf);
                        contact.address.append(&mut adresses);
                    },
                    "website" | "websites" | "url" | "urls" => {
                        let mut websites;
                        (buf, websites) = parse_value(&buf);
                        contact.website.append(&mut websites);
                    },
                    _ => break,
                }

                if peek_next_token(&buf).is_some_and(|x| x == ",") {
                    buf = eat_token(&buf, ",");
                }
            }

            peek_next_token(&buf).expect("expected '}'");
            // content = eat_token(&buf, "}");
            Ok(contact)
        } else { return Err(anyhow!("expected '{{'")) }
    } else { Err(anyhow!("not a valid contact")) }
}


#[cfg(test)]
mod tests {
    use crate::contact::{parse_contact, ContactDefinition};

    #[test]
    fn test_parse_contact_empty() {
        let example_ref: String = "?example{ }".to_string();
        
        let example = parse_contact(example_ref);

        assert!(example.is_ok());
        assert_eq!(example.unwrap(), ContactDefinition{
            id: "example".into(), ..Default::default()
        });
    }
    
    #[test]
    fn test_parse_contact_singles() {
        let example_ref: String = "?example { 
            name: Norm L. Man,
            phone: 10 40 10 37,
            email: some@mail.tld,
            address: \"some actual Adress, City 4048\",
            website: https://some.site.tld
        }".to_string();
        
        let example = parse_contact(example_ref);

        assert!(example.is_ok());
        assert_eq!(example.unwrap(), 
            ContactDefinition {
                name: "Norm L. Man".into(),
                id: "example".into(), 
                phone: vec!["10 40 10 37".into()],
                email: vec!["some@mail.tld".into()],
                address: vec!["some actual Adress, City 4048".into()],
                website: vec!["https://some.site.tld".into()]
            }
        );
    }
    
    #[test]
    fn test_parse_contact_arrays_one_element() {
        let example_ref: String = "?example { 
            name: Norm L. Man,
            phone: [10 40 10 37],
            email: [some@mail.tld],
            address: [\"some actual Adress, City 4048\"],
            website: [https://some.site.tld]
        }".to_string();
        
        let example = parse_contact(example_ref);

        let number : String = "10 40 10 37".into();
        let email  : String = "some@mail.tld".into();
        let address: String = "some actual Adress, City 4048".into();
        let website: String = "https://some.site.tld".into();

        assert!(example.is_ok());
        assert_eq!(example.unwrap(), 
            ContactDefinition {
                id: "example".into(), 
                name: "Norm L. Man".into(),
                phone:   vec![number],
                email:   vec![email],
                address: vec![address],
                website: vec![website],
            }
        );
    }

    #[test]
    fn test_parse_contact_arrays() {
        let example_ref: String = "?example { 
            name: Norm L. Man,
            phone: [10 40 10 37, 10 40 10 37],
            email: [some@mail.tld, some@mail.tld],
            address: [\"some actual Adress, City 4048\", \"some actual Adress, City 4048\"],
            website: [https://some.site.tld, https://some.site.tld]
        }".to_string();
        
        let example = parse_contact(example_ref);

        let number : String = "10 40 10 37".into();
        let email  : String = "some@mail.tld".into();
        let address: String = "some actual Adress, City 4048".into();
        let website: String = "https://some.site.tld".into();

        assert!(example.is_ok());
        assert_eq!(example.unwrap(), 
            ContactDefinition {
                id: "example".into(), 
                name: "Norm L. Man".into(),
                phone:   vec![number.clone() , number.clone() ],
                email:   vec![email.clone()  , email.clone()  ],
                address: vec![address.clone(), address.clone()],
                website: vec![website.clone(), website.clone()]
            }
        );
    }
}
