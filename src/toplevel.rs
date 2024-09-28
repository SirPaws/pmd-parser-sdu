
use config::DEFAULT_FACTBOX_TITLE;
use contact::{parse_contact, ContactDefinition};

use crate::*;

#[derive(Debug, PartialEq)]
pub enum TopLevelSyntax {
    FrontMatter(Frontmatter),
    CodeBlock(String),
    Header(String, usize),
    Image(String, String),
    List(Vec<String>),
    Paragraph(String),
    Quote(Vec<String>),
    ReferenceDefinition(ReferenceDefinition),
    ContactDefinition(ContactDefinition),
    NoteDefinition{id: String, text: String},
    TOC(String),
    PageBreak,
    FactBox{title: String, body: Vec<TopLevelSyntax>},
//  EmbeddedLink(String, String)
}

fn next_line(text: &str) -> &str {
    if text.len() == 0 { return text }
    let mut next_line = 0;
    let mut peek = text.chars().peekable();
    while peek.peek().is_some_and(|x| x.is_whitespace() && x != &'\n') {
        next_line += 1;
        peek.next();
    }
    &text[next_line + 1..]
}

fn string_has_delimeter(text: &str) -> Option<char> {
    for c in text.chars() {
        match c {
            '-'|'.' =>  { return Some(c) }
            _ if c.is_whitespace() => { return Some(c) },
            _ => { continue }
        }
    }
    return None;
}


fn trim_delimeter_start(text: &str) -> &str {
    if text.chars().nth(0).is_some_and(|c| !(c == '.' || c == '-' || c.is_whitespace())) {
        return text;
    }

    let mut num = 0;
    for c in text.chars() {
        match c {
            '-'|'.' => {
                num = num + 1;
            },
            _ if c.is_whitespace() => {
                num = num + 1;
            }
            _ => break,
        }
    }

    &text[num..]
}

// takes the inner string for a meta token
// an example would be #[title], here check would be equal to "title"
// this will also automatically remove whitespace so
// #[      title ] would still be valid.
// if the check string contains whitespace, '.', or '-' it'll be treated as a delimiter marker
// meaning that "last-update" would parse strings like #[last update], #[last.update], 
// #[last - update].
// note: it should only have a single character between meaning "last---update" would be erroneous
// note: all delimiters should be the same, meaning "is-this updated" would be erroneous
fn is_meta(text: &str, check: &str) -> Option<usize> {
    let initial_length = text.len();
    if !text.starts_with("#[") {
        return None;
    }

    let mut text = text[2..].trim_start();

    if let Some(c) = string_has_delimeter(check) {
        let strings : Vec<&str> = check.split(c).collect();

        if !text.starts_with(strings[0]) { return None; }
        
        let mut prev = strings[0];
        for check in strings.iter().skip(1) {
            text = trim_delimeter_start(&text[(prev.len())..]);
            if !text.starts_with(check) { return None; }
            prev = check;
        }

        text = &text[(prev.len())..].trim_start();
    } else {
        if !text.starts_with(check) { return None; }
        text = &text[(check.len())..].trim_start();
    }

    if text.starts_with("]") {
        Some((initial_length - text.len()) + 1)
    } else {
        None
    }
}

fn try_parse_note(content: &str) -> Option<(String, String)> {
    if !content.starts_with("[") { return None }
    if !content[1..].trim_start().starts_with("^") { return None }

    // this is most likely a note definition. as in [^n]: ...
    let remaining = &content[1..].trim_start()[1..];
    let mut note_id = String::new();
    let mut peekable = remaining.chars().peekable();
    while let Some(character) = peekable.peek() && character != &']' {
        note_id.push(*character);
        peekable.next();
    }

    if !peekable.peek().is_some_and(|x| x == &']') { return None }
    peekable.next();
    if !peekable.peek().is_some_and(|x| x == &':') { return None }
    peekable.next();

    note_id = note_id.trim().to_string();
    if note_id.len() == 0 { return None }
    
    let text : String = peekable.collect();

    Some((note_id, text.trim().to_string()))
}

pub fn toplevel_parse_file(file_path: &String) -> Result<Vec<TopLevelSyntax>> {
    toplevel_parse(&fs::read_to_string(file_path)?)
}

fn remove_comments(text: &String) -> String {

    let full_text = text;
    let mut text = &text[..];
    let mut result = String::new();
    while let Some(index) = text.find("%%") {
        if (!text[(index + 2)..].contains("%%")) && (result.len() == 0) {
            return full_text.clone();
        }
        result += &text[..index];
        if let Some(end_index) = (&text[(index + 2)..]).find("%%") {
            text = &(&text[(index + 2)..])[(end_index + 2)..];
            continue;
        } else {
            result += &text[index..];
            break;
        }
    }

    if text.len() > 0 {
        result += text;
    }

    result
}

struct ParseObject {
    is_eating: bool,
    text: String,
    syntax: Vec<TopLevelSyntax>
}
impl ParseObject {
    fn new() -> Self {
        Self {
            is_eating: false,
            text: String::new(),
            syntax: vec![]
        }
    }

    fn eat(&mut self, text: &str) {
        self.is_eating = true;
        self.text += text;
        self.text.push('\n');
    }

    fn consume(&mut self) {
        if self.is_eating {
            let text = self.text.trim();
            if !text.is_empty() {
                self.syntax.push(TopLevelSyntax::Paragraph(format!("{text}\n")));
            }
            self.is_eating = false;
            self.text.clear();
        }
    }

    fn push(&mut self, syntax: TopLevelSyntax) {
        self.consume();
        self.syntax.push(syntax);
    }
}

pub fn find_end_balanced(text: &str, delimiters: (char, char)) -> Option<usize> {

    let mut found_one = false;
    let mut character_count = 0;
    let mut delim_count: usize = 0;
    for character in text.chars() {
        if character == delimiters.0 {
            delim_count = delim_count + 1;
        }
        else if character == delimiters.1 {
            if delim_count == 0 {
                found_one = true;
                break; 
            }
            delim_count = delim_count - 1;
        }
        character_count = character_count + 1;
    }

    if found_one {
        Some(character_count)
    } else {
        None
    }
}

pub fn toplevel_parse(file_content: &String) -> Result<Vec<TopLevelSyntax>> {
    // let matter = Matter::<YAML>::new();
    let (frontmatter, content) = parse_frontmatter(&file_content);

    let mut content = content.to_string();

    let mut object = ParseObject::new();
    /*
    let mut is_eating = false;
    let mut text = String::new();
    let mut toplevel_syntax = Vec::<TopLevelSyntax>::new();
    */
    
    if let Some(frontmatter) = frontmatter {
        object.push(TopLevelSyntax::FrontMatter(frontmatter));
    }

    while !content.is_empty() {
        let current = content.lines().nth(0).context("expected a line where there was none")?;

        if current.len() == 0 {
            object.consume();
            content = next_line(&content[current.len()..]).into();
            continue;
        }

        if current.trim_start().starts_with("%%") {
            let line = &current.trim_start()[2..];
            if let Some(index) = line.find("%%") {
                content = content.trim_start()[2..][(index + 2)..].to_string();
                continue;
            }

            if let Some(index) = content.trim_start()[2..].find("%%") {
                content = content.trim_start()[2..][(index + 2)..].to_string();
                continue;
            }
        }

        if current.starts_with('>') {
            let mut list = Vec::<String>::new();

            let mut line = current;
            while line.starts_with('>') {
                let mut string = &content[1..line.len()];
                string = string.trim();
                list.push(string.into());

                content = next_line(&content[line.len()..]).into();
                let new_line = content.lines().nth(0);
                if new_line.is_none() { break }
                line = new_line.unwrap();
            }

            object.push( TopLevelSyntax::Quote(list));
            continue;
        }

        if current.starts_with("---") && current.ends_with("---") {

            let mut is_line_break = true;
            for character in current.chars() {
                if character.is_whitespace() { continue; }
                if character != '-' {
                    is_line_break = false;
                    break;
                }
            }
            if is_line_break {
                object.push(TopLevelSyntax::PageBreak);
                content = next_line(&content[current.len()..]).into();
                continue;
            }
        }

        if current.starts_with('-') {
            let mut list = Vec::<String>::new();

            let mut line = current;
            while line.starts_with('-') {
                let mut string = &content[1..line.len()];
                string = string.trim();
                list.push(string.into());

                content = next_line(&content[line.len()..]).into();
                let new_line = content.lines().nth(0);
                if new_line.is_none() { break }
                line = new_line.unwrap();
            }

            object.push( TopLevelSyntax::List(list));
            continue;
        }
        
        if current.starts_with("```") {
            let last : usize = content[3..].find("```").context("expected a code block but couldn't find the end")?;

            object.push(TopLevelSyntax::CodeBlock(content[3..last + 3].into()));
            content = next_line(&content[last + 6..]).into();
            continue;
        }
        
        if let Some(n) = is_meta(current, "toc") {
            let text: String = current[n..].trim_start().into();
            object.push(TopLevelSyntax::TOC(text));
            content = next_line(&content[current.len()..]).into();
            continue;
        }
        
        if let Some(n) = is_meta(current, "table-of-content") {
            let text: String = current[n..].trim_start().into();
            object.push(TopLevelSyntax::TOC(text));
            content = next_line(&content[current.len()..]).into();
            continue;
        }
        
        if let Some(n) = is_meta(current, "table-of-contents") {
            let text: String = current[n..].trim_start().into();
            object.push(TopLevelSyntax::TOC(text));
            content = next_line(&content[current.len()..]).into();
            continue;
        }

        if current.starts_with('#') {
            let mut counter = 0;
            while current[counter..].starts_with('#') { 
                counter += 1; 
                if counter >= current.len() { break }
            }

            object.push(TopLevelSyntax::Header( current[counter..].trim_start().into(), counter));

            content = next_line(&content[current.len()..]).into();
            continue;
        }


        if let Some((note_id, note_text)) = try_parse_note(current) {
            object.push(TopLevelSyntax::NoteDefinition { id: note_id, text: note_text });
            content = next_line(&content[current.len()..]).into();
            continue;
        }

        if current.starts_with("[[fact]") || current.starts_with("[[factbox]") {
            let len = if current.starts_with("[[fact]") { "[[fact]".len() } else { "[[factbox]".len() };
            let last = find_end_balanced(&content[len..], ('[', ']')).context("expected a fact box but couldn't find the end")?;

            let mut text_to_parse = String::new();
            let text = &content[len..last + len];
            let title = text.lines().take(1).nth(0);
            for line in text.lines().skip(1) {
                let trimmed = line.trim_start();
                text_to_parse += trimmed;
                text_to_parse.push('\n');
            }

            let body = toplevel_parse(&text_to_parse)?;

            let title = if let Some(text) = title && !text.trim().is_empty() {
                text
            } else {  
                DEFAULT_FACTBOX_TITLE
            }.to_string();
            
            object.push(TopLevelSyntax::FactBox{title, body});
            content = next_line(&content[last + len + 1..]).into();
            continue;
        }
        
        if current.starts_with("[[") || current.starts_with("![[") {
            let count = if current.starts_with("[[") { 2 } else { 3 };
            let text :&str = &current[count..];
            let img_end = text.find(']').context("unable to find ']' for image")?;
            let img = &text[0..img_end];
            let mut remaining_on_line = &text[img_end + 1..];

            let mut alt_text = String::new();
            if remaining_on_line.len() != 0 {
                remaining_on_line = remaining_on_line.trim_start();
                if remaining_on_line.starts_with(']') {
                    object.push(TopLevelSyntax::Image(img.into(), "".into()));
                    content = next_line(&content[current.len()..]).into();
                    continue;
                }

                if let Some(index) = remaining_on_line.find(']') {
                    alt_text = remaining_on_line[0..index].into();
                    object.push(TopLevelSyntax::Image(img.into(), alt_text));
                    content = next_line(&content[current.len()..]).into();
                    continue;
                }

                if remaining_on_line.len() > 0 {
                    alt_text += remaining_on_line.into();
                    alt_text += "\n";
                }
            }

            let image_text = String::from(img);
            let end_index = content.find(']').context("unable to find ']' for image")?;
            

            content = next_line(&content[current.len()..]).into();
            alt_text += &content[0..end_index];
            object.push(
                // if image_text.starts_with("http") {
                //     TopLevelSyntax::EmbeddedLink(image_text, alt_text)
                // } else {
                    TopLevelSyntax::Image(image_text, alt_text)
                // }
            );
            content = next_line(&content[end_index + 1..]).into();
            continue;
        }

        if current.starts_with("£") && current.chars().nth(1).is_some_and(|c| return char::is_alphabetic(c) || c == '-') {

            // todo take only the 
            
            let end = (&content).find('}').expect("could not find the end of citation");
            let citation = parse_reference(content[0..(end+1)].to_string())?;
            object.push(TopLevelSyntax::ReferenceDefinition(citation));
            content = content[(end + 1)..].to_string();
            continue;
        }
        
        if current.starts_with("?") && current.chars().nth(1).is_some_and(|c| return char::is_alphabetic(c) || c == '-') {
            let end = (&content).find('}').expect("could not find the end of contact");
            let contact = parse_contact(content[0..(end+1)].to_string())?;
            object.push(TopLevelSyntax::ContactDefinition(contact));
            content = content[(end + 1)..].to_string();
            continue;
        }
        

        object.eat(&content[0..current.len()]);
        content = next_line(&content[current.len()..]).into();
    }

    object.consume();

    for element in &mut object.syntax {
        match element {
            TopLevelSyntax::Paragraph(text) => {
                if !text.contains("%%") { continue }
                *text = remove_comments(text);
            }
            _ => { continue }
        }
    }


    Ok(object.syntax)
}

#[cfg(test)]
mod tests {
    use crate::toplevel::*;

    #[test]
    fn test_next_line() {
        let text = "     \nhello\ngoodbye"; 
        assert!( next_line(text) == "hello\ngoodbye");
        assert!( next_line(&next_line(text)[5..]) == "goodbye");
    }

    #[test]
    fn test_paragraph() {
        let text = "hello world\nthis is some\ntext that is split, between\nmultiple lines\n".to_string();

        let result = toplevel_parse(&text);
        assert!(result.is_ok());
        let syntax = result.unwrap();
        assert_eq!(syntax, vec![TopLevelSyntax::Paragraph(text)])
    }
    
    #[test]
    fn test_adds_newline() {
        let text = "hello world".to_string();

        let result = toplevel_parse(&text);
        assert!(result.is_ok());
        let syntax = result.unwrap();
        assert_eq!(syntax, vec![TopLevelSyntax::Paragraph("hello world\n".into())])
    }

    #[test]
    fn test_remove_paragraph_comment_start() {
        let text = "%%this is a comment that will be removed%%this won't be removed".to_string();

        let result = toplevel_parse(&text);
        assert!(result.is_ok());
        let syntax = result.unwrap();
        assert_eq!(syntax, vec![TopLevelSyntax::Paragraph("this won't be removed\n".into())])
    }
    
    #[test]
    fn test_remove_paragraph_comment_middle() {
        let text = "Hi %%this is a comment that will be removed%% this won't be removed".to_string();

        let result = toplevel_parse(&text);
        assert!(result.is_ok());
        let syntax = result.unwrap();
        assert_eq!(syntax, vec![TopLevelSyntax::Paragraph("Hi  this won't be removed\n".into())])
    }
    
    #[test]
    fn test_remove_paragraph_comment_end() {
        let text = "this won't be removed%%this is a comment that will be removed%%".to_string();

        let result = toplevel_parse(&text);
        assert!(result.is_ok());
        let syntax = result.unwrap();
        assert_eq!(syntax, vec![TopLevelSyntax::Paragraph("this won't be removed\n".into())])
    }

    #[test]
    fn test_paragraph_order() {
        let text = "here is a paragraph with some text\n# and here is a header".to_string();
        
        let result = toplevel_parse(&text);
        assert!(result.is_ok());
        let syntax = result.unwrap();
        assert_eq!(syntax, vec![
            TopLevelSyntax::Paragraph("here is a paragraph with some text\n".into()),
            TopLevelSyntax::Header("and here is a header".into(), 1),
        ])
    }
    
    // #[test]
    // fn test_title() {
    //     let text = "#[title] this is a title".to_string();
    // 
    //     let result = toplevel_parse(&text);
    //     assert!(result.is_ok());
    //     let syntax = result.unwrap();
    //     assert_eq!(syntax, vec![TopLevelSyntax::Title("this is a title".into())])
    // }
    
    // #[test]
    // fn test_meta_whitespace() {
    //     let text = "#[      title  ] this is a title".to_string();
    // 
    //     let result = toplevel_parse(&text);
    //     assert!(result.is_ok());
    //     let syntax = result.unwrap();
    //     assert_eq!(syntax, vec![TopLevelSyntax::Title("this is a title".into())])
    // }
    // 
    // #[test]
    // fn test_subtitle() {
    //     let text = "#[subtitle] this is a subtitle".to_string();
    // 
    //     let result = toplevel_parse(&text);
    //     assert!(result.is_ok());
    //     let syntax = result.unwrap();
    //     assert_eq!(syntax, vec![TopLevelSyntax::Subtitle("this is a subtitle".into())])
    // }

    // #[test]
    // fn test_frontmatter() {
    //     let text = "---\ntest: text\n---".to_string();
    //     let result = toplevel_parse(&text);
    //     assert!(result.is_ok());
    //     let syntax = result.unwrap();
    //     
    //     assert_eq!(syntax, vec![TopLevelSyntax::FrontMatter(Pod::Hash(HashMap::from([("test".to_string(), Pod::String("text".to_string()))])))])
    // }
    // 
    // #[test]
    // fn test_frontmatter_with_paragraph() {
    //     let text = "---\ntest: text\n---\nhey there".to_string();
    //     let result = toplevel_parse(&text);
    //     assert!(result.is_ok());
    //     let syntax = result.unwrap();
    //     
    //     assert_eq!(syntax, 
    //                vec![
    //                 TopLevelSyntax::FrontMatter(
    //                     Pod::Hash(HashMap::from([("test".to_string(), Pod::String("text".to_string()))]))),
    //                     TopLevelSyntax::Paragraph("hey there\n".to_string())
    //                ])
    // }
    
    // #[test]
    // fn test_notes_title_dash() {
    //     let text = "#[notes-title] Notes".to_string();
    // 
    //     let result = toplevel_parse(&text);
    //     assert!(result.is_ok());
    //     let syntax = result.unwrap();
    //     assert_eq!(syntax, vec![TopLevelSyntax::NotesTitle("Notes".into())])
    // }
    // 
    // #[test]
    // fn test_notes_title_space() {
    //     let text = "#[notes title] Notes".to_string();
    // 
    //     let result = toplevel_parse(&text);
    //     assert!(result.is_ok());
    //     let syntax = result.unwrap();
    //     assert_eq!(syntax, vec![TopLevelSyntax::NotesTitle("Notes".into())])
    // }
    
    // #[test]
    // fn test_notes_title_dot() {
    //     let text = "#[notes.title] Notes".to_string();
    // 
    //     let result = toplevel_parse(&text);
    //     assert!(result.is_ok());
    //     let syntax = result.unwrap();
    //     assert_eq!(syntax, vec![TopLevelSyntax::NotesTitle("Notes".into())])
    // }
    // 
    // #[test]
    // fn test_bibliography_title() {
    //     let text = "#[bibliography-title] Refs".to_string();
    // 
    //     let result = toplevel_parse(&text);
    //     assert!(result.is_ok());
    //     let syntax = result.unwrap();
    //     assert_eq!(syntax, vec![TopLevelSyntax::BibliographyTitle("Refs".into())])
    // }
    // 
    // #[test]
    // fn test_last_update() {
    //     let text = "#[last-update] Refs".to_string();
    // 
    //     let result = toplevel_parse(&text);
    //     assert!(result.is_ok());
    //     let syntax = result.unwrap();
    //     assert_eq!(syntax, vec![TopLevelSyntax::LastUpdateDate(PmdDate::String("Refs".into()))])
    // }
    // 
    // #[test]
    // fn test_last_update_alternate_spelling() {
    //     let text = "#[last-updated] Refs".to_string();
    // 
    //     let result = toplevel_parse(&text);
    //     assert!(result.is_ok());
    //     let syntax = result.unwrap();
    //     assert_eq!(syntax, vec![TopLevelSyntax::LastUpdateDate(PmdDate::String("Refs".into()))])
    // }
    
    #[test]
    fn test_toc() {
        let text = "#[toc] Refs".to_string();

        let result = toplevel_parse(&text);
        assert!(result.is_ok());
        let syntax = result.unwrap();
        assert_eq!(syntax, vec![TopLevelSyntax::TOC("Refs".into())])
    }
    
    #[test]
    fn test_toc_alternate_spelling0() {
        let text = "#[table of content] Refs".to_string();

        let result = toplevel_parse(&text);
        assert!(result.is_ok());
        let syntax = result.unwrap();
        assert_eq!(syntax, vec![TopLevelSyntax::TOC("Refs".into())])
    }
    
    #[test]
    fn test_toc_alternate_spelling1() {
        let text = "#[table of contents] Refs".to_string();

        let result = toplevel_parse(&text);
        assert!(result.is_ok());
        let syntax = result.unwrap();
        assert_eq!(syntax, vec![TopLevelSyntax::TOC("Refs".into())])
    }
    
    // #[test]
    // fn test_banner() {
    //     let text = "#[banner] this/banner/path.png".to_string();
    // 
    //     let result = toplevel_parse(&text);
    //     assert!(result.is_ok());
    //     let syntax = result.unwrap();
    //     assert_eq!(syntax, vec![TopLevelSyntax::Banner("this/banner/path.png".into())])
    // }

    #[test]
    fn test_header() {
        let text = "# this is a header".to_string();

        let result = toplevel_parse(&text);
        assert!(result.is_ok());
        let syntax = result.unwrap();
        assert_eq!(syntax, vec![TopLevelSyntax::Header("this is a header".into(), 1)])
    }

    #[test]
    fn test_codeblock() {
        let text = "```\ncodeblock```".to_string();

        let result = toplevel_parse(&text);
        assert!(result.is_ok());
        let syntax = result.unwrap();
        assert_eq!(syntax, vec![TopLevelSyntax::CodeBlock("\ncodeblock".into())])
    }
    
    #[test]
    fn test_codeblock_with_langcode() {
        let text = "```c\ncodeblock```".to_string();

        let result = toplevel_parse(&text);
        assert!(result.is_ok());
        let syntax = result.unwrap();
        assert_eq!(syntax, vec![TopLevelSyntax::CodeBlock("c\ncodeblock".into())])
    }

    /*
    #[test]
    fn test_url() {
        let text = "[[http://some.link]]".to_string();

        let result = toplevel_parse(&text);
        assert!(result.is_ok());
        let syntax = result.unwrap();
        assert_eq!(syntax, vec![TopLevelSyntax::EmbeddedLink("http://some.link".into(), "".into())])
    }*/

    #[test]
    fn test_url_https() {
        let text = "[[https://some.link]]".to_string();

        let result = toplevel_parse(&text);
        assert!(result.is_ok());
        let syntax = result.unwrap();
        assert_eq!(syntax, vec![TopLevelSyntax::Image("https://some.link".into(), "".into())])
    }
    
    #[test]
    fn test_url_with_alt_text() {
        let text = "[[http://some.link] with alt text]".to_string();

        let result = toplevel_parse(&text);
        assert!(result.is_ok());
        let syntax = result.unwrap();
        assert_eq!(syntax, vec![TopLevelSyntax::Image("http://some.link".into(), "with alt text".into())])
    }
    
    #[test]
    fn test_image() {
        let text = "[[image.png]]".to_string();

        let result = toplevel_parse(&text);
        assert!(result.is_ok());
        let syntax = result.unwrap();
        assert_eq!(syntax, vec![TopLevelSyntax::Image("image.png".into(), "".into())])
    }
    
    #[test]
    fn test_image_with_alt_text() {
        let text = "[[image.png] with alt text]".to_string();

        let result = toplevel_parse(&text);
        assert!(result.is_ok());
        let syntax = result.unwrap();
        assert_eq!(syntax, vec![TopLevelSyntax::Image("image.png".into(), "with alt text".into())])
    }
    
    #[test]
    fn test_list() {
        let text = "- first\n- second\n- third\n".to_string();

        let result = toplevel_parse(&text);
        assert!(result.is_ok());
        let syntax = result.unwrap();
        assert_eq!(syntax, vec![TopLevelSyntax::List(vec!["first".into(), "second".into(), "third".into()])])
    }
    
    #[test]
    fn test_quote() {
        let text = "> hello world\n> this is some\n> text that is split, between\n> multiple lines\n".to_string();

        let result = toplevel_parse(&text);
        assert!(result.is_ok());
        let syntax = result.unwrap();
        assert_eq!(syntax, vec![TopLevelSyntax::Quote(vec![
            "hello world".into(), 
            "this is some".into(), 
            "text that is split, between".into(), 
            "multiple lines".into()
        ])])
    }
    
    #[test]
    fn test_note() {
        let text = "[ ^ 0 ]: this is a random note".to_string();

        let result = toplevel_parse(&text);
        assert!(result.is_ok());
        let syntax = result.unwrap();
        assert_eq!(syntax, vec![TopLevelSyntax::NoteDefinition { id: "0".into(), text: "this is a random note".into() }]);
    }

    #[test]
    fn test_single_line_comment() {
        let text = "          %% this is a comment %%        ".to_string();

        let result = toplevel_parse(&text);
        assert!(result.is_ok());
        let syntax = result.unwrap();
        assert_eq!(syntax, vec![]);
    }
    #[test]
    fn test_single_multiline_comment() {
        let text = "          %%this\nis\na\nmultiline\ncomment%%        ".to_string();

        let result = toplevel_parse(&text);
        assert!(result.is_ok());
        let syntax = result.unwrap();
        assert_eq!(syntax, vec![]);
    }
    
    #[test]
    fn test_comment_overrun() {
        let text = "          %%this\nis\na\nmultiline\ncomment%%this is text".to_string();

        let result = toplevel_parse(&text);
        assert!(result.is_ok());
        let syntax = result.unwrap();
        assert_eq!(syntax, vec![TopLevelSyntax::Paragraph("this is text\n".into())]);
    }

    #[test]
    fn test_reference_overrun() {
        let text: String = "£example { 
            title: Simulacra and Simulation,
            author: Jean Baudrillard,
            publisher: University of Michigan Press,
            year: 1994,
            pages: 176,
            esbn: 0-472-06521-1,
        }here's a reference and a paragraph".to_string();
        
        let result = toplevel_parse(&text);
        assert!(result.is_ok());
        let syntax = result.unwrap();

        assert_eq!(syntax, vec![TopLevelSyntax::ReferenceDefinition(ReferenceDefinition{ 
            id: "example".into(), 
            authors: vec!["Jean Baudrillard".into()],
            editors: vec![],
            translators: vec![],
            title: "Simulacra and Simulation".into(),
            description: "".into(),
            container_title: "".into(),
            publisher: "University of Michigan Press".into(),
            date: PmdDate::Split{ day: None, month: None, year: Some(1994) },
            date_retrieved: PmdDate::None,
            volume: "".into(),
            edition: "".into(),
            version: "".into(),
            issue: "".into(),
            pages: "176".into(),
            link: "".into(),
            doi: "".into(),
            esbn: "0-472-06521-1".into()
        }), TopLevelSyntax::Paragraph("here's a reference and a paragraph\n".into())]);
    }
}
