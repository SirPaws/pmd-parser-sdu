use color_print::cprintln;
use contact::ContactDefinition;
use serde_yaml::Value;
// use gray_matter::Pod;

use crate::*;
use crate::config::{DEFAULT_URL, DEFAULT_DATA_DIR, DEFAULT_BLOG_DIR};

#[derive(Debug, PartialEq, Clone)]
pub struct TableOfContent {
    pub title:   String,
    pub index:   usize,
    pub max_depth: usize,
    pub headers: Vec<(Box<BlogBody>, usize)>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BlogHeader {
    pub title: String,
    pub subtitle: String,
    pub banner: String,
    pub url: String,
    pub data_dir: String,
    pub blog_dir: String,
    pub date_written: PmdDate,
    pub last_update: PmdDate,
    pub toc: Option<TableOfContent>,
    pub bibliography_title: String,
    pub notes_title: String,
    pub frontmatter: Option<Frontmatter>,
}

impl BlogHeader {
    fn default() -> Self {
        Self {
            title: "".into(),
            subtitle: "".into(),
            banner: "".into(),
            url: DEFAULT_URL.into(),
            data_dir: DEFAULT_DATA_DIR.into(),
            blog_dir: DEFAULT_BLOG_DIR.into(),
            date_written: PmdDate::None,
            last_update: PmdDate::None,
            toc:      None,
            bibliography_title: "References".into(),
            notes_title: "Notes".into(),
            frontmatter: None,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Alternative {
    pub base: Box<BlogBody>,
    pub alt:  Box<BlogBody>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Span {
    pub elements: Vec<BlogBody>
}

#[derive(Debug, PartialEq, Clone)]
pub enum BlogBody {
    Hoverable(Alternative),
    Styled(Alternative),
    Link(Alternative),
    Header(Box<BlogBody>, usize),
    Italics(Box<BlogBody>),
    Bold(Box<BlogBody>),
    InlineCode(String),
    CodeBlock(String),
    Image(String, String),
    // EmbeddedLink(String, String),
    Quote(Vec<BlogBody>),
    List(Vec<BlogBody>),
    Paragraph(Box<BlogBody>),
    Text(String),
    Span(Span),
    Citation(String),
    Note(String),
    TOCLocationMarker,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PawsMarkdown {
    pub header: BlogHeader,
    pub references: HashMap<String, ReferenceDefinition>,
    pub notes: HashMap::<String, BlogBody>,
    pub body  : Vec<BlogBody>
}

pub fn text_parse(text: &String) -> Result<Box<BlogBody>> {

    let mut body = Vec::<BlogBody>::new();
    let mut buffer = String::new();
    let mut peekable = text.chars().peekable();
    while let Some(character) = peekable.peek() {
        match character {
            '\\' => {
                peekable.next(); 
                let mut copy = peekable.clone();
                if let Some(&escaped_character) = copy.peek() {
                    copy.next();
                    match escaped_character {
                        '%'|'£' => { 
                            buffer.push(escaped_character);
                            if copy.peek().is_some_and(|&possible_brace| possible_brace == '[') {
                                peekable.next();
                                buffer.push('[');
                            }
                        },
                        _ => {
                            buffer.push(escaped_character);
                        }
                    }
                } else {
                    buffer.push('\\');
                }
            },
            '£'|'%' => {
                if buffer.len() != 0 {
                    body.push(BlogBody::Text(buffer));
                    buffer = String::new();
                }

                let start_char = *character;
                let make_object = |base: &String, alt: &String|
                    anyhow::Ok(if start_char == '%' { 
                        BlogBody::Hoverable(
                            Alternative{base: text_parse(base)?, alt: text_parse(alt)?}
                        )
                    } else {
                        BlogBody::Styled(
                            Alternative{base: text_parse(base)?, alt: text_parse(alt)?}
                        )
                    });
                
                let search_begin_char = if start_char == '%' { '[' } else { '{' };
                let search_end_char   = if start_char == '%' { ']' } else { '}' };

                peekable.next();
                if peekable.peek().is_some_and(|&brace| brace == search_begin_char) {
                    let mut base = String::new(); 
                    peekable.next();
                    let mut depth = 0;
                    let mut end  = peekable.clone();
                    while !(end.peek() == Some(&search_end_char) && depth == 0) {
                        let character = end.next().context("expected ']'")?;
                        if      character == search_begin_char { depth += 1 }
                        else if character == search_end_char { depth -= 1 }
                        base.push(character);
                    }
                    end.next();
                    let mut alt = String::new();
                    // this is unreadable
                    body.push(make_object(&base, 
                        if end.peek() == Some(&'(') {
                            end.next();
                            while !(end.peek() == Some(&')') && depth == 0) {
                                let character = end.next().context("expected ')'")?;
                                match character {
                                    '(' => depth += 1,
                                    ')' => if depth != 0 { depth -= 1 } else {},
                                    _   => {}
                                }
                                alt.push(character);
                            }
                            end.next();
                            peekable = end.clone();
                            &alt
                        }
                        else {
                            peekable = end.clone();
                            &alt
                        })?);
                } else {
                    buffer.push(start_char);
                }
                continue;
            },
            '[' => {
                if buffer.len() != 0 {
                    body.push(BlogBody::Text(buffer));
                    buffer = String::new();
                }

                let mut base = String::new(); 
                peekable.next();

                let mut depth = 0;
                let mut end  = peekable.clone();
                while !(end.peek() == Some(&']') && depth == 0) {
                    let character = end.next().context("expected ']'")?;
                    match character {
                        '[' => depth += 1,
                        ']' => if depth != 0 { depth -= 1 } else {},
                        _   => {}
                    }
                    base.push(character);
                }

                if base.starts_with('£') && base.trim_start().chars().nth(1).is_some_and(|x| x.is_alphabetic() || x == '-') {
                    // this is a citation
                    let citation : String = base.chars().skip(1).collect();
                    body.push(BlogBody::Citation(citation));
                    end.next();
                    peekable = end.clone();
                    continue;
                }
                
                
                if base.starts_with('^') && base.len() > 1 {
                    // this is a citation
                    let citation : String = base.chars().skip(1).collect();
                    body.push(BlogBody::Note(citation));
                    end.next();
                    peekable = end.clone();
                    continue;
                }

                end.next();
                let mut alt = String::new();
                // this is unreadable
                body.push(BlogBody::Link(Alternative{base: text_parse(&base)?, 
                    alt: text_parse(if end.peek() == Some(&'(') {
                        end.next();
                        while !(end.peek() == Some(&')') && depth == 0) {
                            let character = end.next().context("expected ')'")?;
                            match character {
                                '(' => depth += 1,
                                ')' => if depth != 0 { depth -= 1 } else {},
                                _   => {}
                            }
                            alt.push(character);
                        }
                        end.next();
                        peekable = end.clone();
                        &alt
                    }
                    else {
                        peekable = end.clone();
                        &alt
                    })?}));
                continue;
            },
            '`' => {
                if buffer.len() != 0 {
                    body.push(BlogBody::Text(buffer));
                    buffer = String::new();
                }

                let mut base = String::new(); 
                peekable.next();
                let mut end  = peekable.clone();
                while end.peek() != Some(&'`') {
                    let character = end.next().context("expected ']'")?;
                    base.push(character);
                }
                end.next();
                peekable = end.clone();
                body.push(BlogBody::InlineCode(base));
                continue;
            },
            '*' => {
                if buffer.len() != 0 {
                    body.push(BlogBody::Text(buffer));
                    buffer = String::new();
                }

                peekable.next();
                if peekable.peek() == Some(&'*') {
                    peekable.next();
                    let mut depth = 0;
                    let mut result = String::new();
                    while peekable.peek().is_some() {
                        if peekable.peek() == Some(&'*') {
                            peekable.next();
                            if peekable.peek() == Some(&'*') && depth == 0 {
                                break;
                            } else {
                                if depth == 0 { depth += 1; } else { depth -= 1; }
                            }
                            result.push('*');
                            continue;
                        }
                        result.push(peekable.next().unwrap());
                    }
                    if peekable.peek() == Some(&'*') {
                        peekable.next();
                    }
                    body.push(BlogBody::Bold(text_parse(&result)?))
                } else {
                    let mut result = String::new();
                    while peekable.peek() != Some(&'*') {
                        if peekable.peek().is_none() { break }
                        result.push(peekable.next().unwrap());
                    }
                    if peekable.peek() == Some(&'*') {
                        peekable.next();
                    }
                    body.push(BlogBody::Italics(text_parse(&result)?))
                }
                continue;
                
            },
            _   => { buffer.push(*character); },
        }

        peekable.next();
    }

    if buffer.len() != 0 {
        body.push(BlogBody::Text(buffer));
    }

    match body.len() {
        0 => Ok(Box::new(BlogBody::Span(Span{elements: vec![]}))),
        1 => Ok(Box::new(body[0].clone())),
        _ => Ok(Box::new(BlogBody::Span(Span{elements: body}))),
    }
}

fn get_url(data: &Frontmatter) -> Option<String> {
    if let Some(url) = data["url"].as_string() {
        Some(url)
    } else if let Some(url) = data["base_url"].as_string() {
        Some(url)
    } else if let Some(url) = data["base url"].as_string() {
        Some(url)
    } else if let Some(url) = data["base-url"].as_string() {
        Some(url)
    } else {
        None
    }
}

fn get_data_dir(data: &Frontmatter) -> Option<String> {
    if let Some(url) = data["data"].as_string() {
        Some(url)
    } else if let Some(url) = data["data_dir"].as_string() {
        Some(url)
    } else if let Some(url) = data["data-dir"].as_string() {
        Some(url)
    } else if let Some(url) = data["data dir"].as_string() {
        Some(url)
    } else {
        None
    }
}

fn get_blog_dir(data: &Frontmatter) -> Option<String> {
    if let Some(url) = data["blog"].as_string() {
        Some(url)
    } else if let Some(url) = data["blog_dir"].as_string() {
        Some(url)
    } else if let Some(url) = data["blog-dir"].as_string() {
        Some(url)
    } else if let Some(url) = data["blog dir"].as_string() {
        Some(url)
    } else {
        None
    }
}

fn get_date(data: &Frontmatter) -> Option<String> {
    if let Some(date) = data["date"].as_string() {
        Some(date)
    } else if let Some(date) = data["date-written"].as_string() {
        Some(date)
    } else if let Some(date) = data["date_written"].as_string() {
        Some(date)
    } else if let Some(date) = data["date written"].as_string() {
        Some(date)
    } else {
        None
    }
}

fn get_last_update(data: &Frontmatter) -> Option<String> {
    if let Some(date) = data["last-update"].as_string() {
        Some(date)
    } else if let Some(date) = data["last_update"].as_string() {
        Some(date)
    } else if let Some(date) = data["last update"].as_string() {
        Some(date)
    } else if let Some(date) = data["last-updated"].as_string() {
        Some(date)
    } else if let Some(date) = data["last_updated"].as_string() {
        Some(date)
    } else if let Some(date) = data["last updated"].as_string() {
        Some(date)
    } else {
        None
    }
}

fn get_bibliography_title(data: &Frontmatter) -> Option<String> {
    if let Some(title) = data["bibliography-title"].as_string() {
        Some(title)
    } else if let Some(title) = data["references-title"].as_string() {
        Some(title)
    } else if let Some(title) = data["sources-title"].as_string() {
        Some(title)
    } else if let Some(title) = data["bibliography title"].as_string() {
        Some(title)
    } else if let Some(title) = data["references title"].as_string() {
        Some(title)
    } else if let Some(title) = data["sources title"].as_string() {
        Some(title)
    } else if let Some(title) = data["bibliography_title"].as_string() {
        Some(title)
    } else if let Some(title) = data["references_title"].as_string() {
        Some(title)
    } else if let Some(title) = data["sources_title"].as_string() {
        Some(title)
    } else {
        None
    }
}

pub fn file_parse(file_path: &String) -> Result<PawsMarkdown> {
    let toplevel_syntax = toplevel_parse_file(file_path)?; 

    let mut notes      = HashMap::<String, BlogBody>::new();
    let mut references = HashMap::<String, ReferenceDefinition>::new();
    let mut header: BlogHeader = BlogHeader::default();
    let mut body = Vec::<BlogBody>::new();
    for elem in &toplevel_syntax {
        match elem {
            TopLevelSyntax::FrontMatter(frontmatter) => { // (data)   => {
                header.frontmatter = Some(frontmatter.clone());
                // println!("FRONTMATTER!!! {:?}", data);
            }
            // TopLevelSyntax::LastUpdateDate(date) => { header.date = date.clone(); },
            // TopLevelSyntax::Subtitle(text)      => { header.subtitle = text.to_string();                                },
            // TopLevelSyntax::Title(text)         => { header.title    = text.to_string();                                },
            // TopLevelSyntax::Banner(text)        => { header.banner   = text.to_string();                                },
            TopLevelSyntax::CodeBlock(block)    => { body.push(BlogBody::CodeBlock(block.to_string()));            },
            TopLevelSyntax::Image(img, alt)     => { body.push(BlogBody::Image(img.to_string(), alt.to_string())); },
            // TopLevelSyntax::EmbeddedLink(img, alt) => { body.push(BlogBody::EmbeddedLink(img.to_string(), alt.to_string())); },
            TopLevelSyntax::Header(text, level) => { body.push(BlogBody::Header(text_parse(&text)?, *level));      },
            TopLevelSyntax::List(list)          => {
                let mut result = Vec::<BlogBody>::new();
                for elem in list {
                    result.push(Box::into_inner(text_parse(&elem)?))
                }
                body.push(BlogBody::List(result));
            }
            TopLevelSyntax::Paragraph(text)     => { body.push(BlogBody::Paragraph(text_parse(&text)?));           },
            TopLevelSyntax::Quote(list)         => { 
                let mut result = Vec::<BlogBody>::new();
                for elem in list {
                    result.push(Box::into_inner(text_parse(&elem)?))
                }
                body.push(BlogBody::Quote(result));
            },
            TopLevelSyntax::ReferenceDefinition(reference) => { references.insert(reference.id.clone(), reference.clone()); },
            TopLevelSyntax::NoteDefinition { id, text } => {
                notes.insert(id.clone(), Box::into_inner(text_parse(&text)?));
            }
            TopLevelSyntax::TOC(title) => {
                if header.toc.is_none() {
                    header.toc = Some(TableOfContent{ title: title.clone(), index: body.len(), headers: vec![], max_depth: 1,});
                    body.push(BlogBody::TOCLocationMarker);
                }
            },
            // TopLevelSyntax::NotesTitle(title) => { header.notes_title = title.clone(); },
            // TopLevelSyntax::BibliographyTitle(title) => { header.bibliography_title = title.clone() },
        };
    }

    if header.toc.is_some() {
        let toc = header.toc.as_mut().unwrap();
        for (i, item) in body.iter().enumerate() {
            if i < toc.index { continue; }
            if let &BlogBody::Header(text, depth) = &item {
                if depth > &toc.max_depth {
                    toc.max_depth = *depth;
                }
                toc.headers.push((text.clone(), depth.clone()));
            }
        }

        if notes.len() > 0 {
            toc.headers.push((Box::new(BlogBody::Text(header.notes_title.clone())), 1))
        }


        if references.len() > 0 {
            toc.headers.push((Box::new(BlogBody::Text(header.bibliography_title.clone())), 1))
        }
    }

    if let Some(frontmatter) = &header.frontmatter {
        if let Some(title) = frontmatter["title"].as_string() {
            header.title = title;
        } else {
            cprintln!("<r>error:</> Document '{}' is missing a title, see 'pmd explain frontmatter'", file_path);
        }
        
        if let Some(subtitle) = frontmatter["subtitle"].as_string() {
            header.subtitle = subtitle;
        }
        
        if let Some(banner) = frontmatter["banner"].as_string() {
            header.banner = banner;
        }
        
        if let Some(title) = frontmatter["notes-title"].as_string() {
            header.notes_title = title;
        }

        if let Some(title) = get_bibliography_title(frontmatter) {
            header.bibliography_title = title;
        }

        if let Some(date) = get_date(frontmatter) {
            header.date_written = PmdDate::String(date);
        } else {
            cprintln!("<r>error:</> Document '{}' is missing a date, see 'pmd explain frontmatter'", file_path);
        }
        
        if let Some(update) = get_last_update(frontmatter) {
            header.last_update = PmdDate::String(update);
        }
        
        if let Some(url) = get_url(frontmatter) {
            header.url = url;
        }
        if let Some(data_dir) = get_data_dir(frontmatter) {
            header.data_dir = data_dir;
        }
        if let Some(blog_dir) = get_blog_dir(frontmatter) {
            header.blog_dir = blog_dir;
        }
    } else {
        cprintln!("<r>error:</> Document '{}' is missing frontmatter, see 'pmd explain frontmatter'", file_path);
    }

    Ok(PawsMarkdown { header, references, notes, body })
}


#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_parse_remove_escaped() {
        let text: String = "\\[]".into();
        let result = text_parse(&text);
        assert!(result.is_ok());
        let inner = Box::into_inner(result.unwrap());
        assert!(inner == BlogBody::Text("[]".into()));
    }
    
    #[test]
    fn test_parse_remove_escaped_embedding() {
        let text: String = "\\%[]".into();
        let result = text_parse(&text);
        assert!(result.is_ok());
        let inner = Box::into_inner(result.unwrap());
        assert!(inner == BlogBody::Text("%[]".into()));
    }
    
    #[test]
    fn test_parse_hover() {
        let text: String = "%[abc](def)".into();
        let result = text_parse(&text);
        assert!(result.is_ok());
        let inner = Box::into_inner(result.unwrap());
        assert!(inner == BlogBody::Hoverable(Alternative{ base: Box::new(BlogBody::Text("abc".into())), alt: Box::new(BlogBody::Text("def".into()))}));
    }
    
    #[test]
    fn test_parse_styling() {
        let text: String = "£{abc}(def)".into();
        let result = text_parse(&text);
        assert!(result.is_ok());
        let inner = Box::into_inner(result.unwrap());
        assert!(inner == BlogBody::Styled(Alternative{ base: Box::new(BlogBody::Text("abc".into())), alt: Box::new(BlogBody::Text("def".into()))}));
    }
    
    #[test]
    fn test_parse_hover_with_inner_styling_left() {
        let text: String = "%[£{style}(text)](alternative)".into();
        let result = text_parse(&text);
        assert!(result.is_ok());
        let inner = Box::into_inner(result.unwrap());
        assert!(inner == BlogBody::Hoverable(Alternative{ 
            base: Box::new(BlogBody::Styled(
                Alternative {
                    base: Box::new(BlogBody::Text("style".into())),
                    alt: Box::new(BlogBody::Text("text".into()))
                }
            )), 
            alt: Box::new(BlogBody::Text("alternative".into()))
        }));
    }
    
    #[test]
    fn test_parse_hover_with_inner_styling_right() {
        let text: String = "%[base](£{style}(text))".into();
        let result = text_parse(&text);
        assert!(result.is_ok());
        let inner = Box::into_inner(result.unwrap());
        assert!(inner == BlogBody::Hoverable(Alternative{ 
            base: Box::new(BlogBody::Text("base".into())),
            alt: Box::new(BlogBody::Styled(
                Alternative {
                    base: Box::new(BlogBody::Text("style".into())),
                    alt: Box::new(BlogBody::Text("text".into()))
                }
            )), 
        }));
    }

    #[test]
    fn test_parse_link() {
        let text: String = "[abc](def)".into();
        let result = text_parse(&text);
        assert!(result.is_ok());
        let inner = Box::into_inner(result.unwrap());
        assert!(inner == BlogBody::Link(Alternative{ base: Box::new(BlogBody::Text("abc".into())), alt: Box::new(BlogBody::Text("def".into()))}));
    }
    
    #[test]
    fn test_parse_link_with_styling() {
        let text: String = "[link](£{style}(text))".into();
        let result = text_parse(&text);
        assert!(result.is_ok());
        let inner = Box::into_inner(result.unwrap());
        assert!(inner == BlogBody::Link(Alternative{ 
            base: Box::new(BlogBody::Text("link".into())),
            alt: Box::new(BlogBody::Styled(
                Alternative {
                    base: Box::new(BlogBody::Text("style".into())),
                    alt: Box::new(BlogBody::Text("text".into()))
                }
            )), 
        }));
    }

    #[test]
    fn test_parse_inline_code() {
        let text: String = "`here's some code`".into();
        let result = text_parse(&text);
        assert!(result.is_ok());
        let inner = Box::into_inner(result.unwrap());
        assert!(inner == BlogBody::InlineCode("here's some code".into()))
    }

    #[test]
    fn test_parse_italics() {
        let text: String = "*italics*".into();
        let result = text_parse(&text);
        assert!(result.is_ok());
        let inner = Box::into_inner(result.unwrap());
        assert!(inner == BlogBody::Italics(Box::new(BlogBody::Text("italics".into()))))
    }
    
    #[test]
    fn test_parse_bold() {
        let text: String = "**bold**".into();
        let result = text_parse(&text);
        assert!(result.is_ok());
        let inner = Box::into_inner(result.unwrap());
        assert!(inner == BlogBody::Bold(Box::new(BlogBody::Text("bold".into()))))
    }
    
    #[test]
    fn test_parse_bold_and_italics() {
        let text: String = "***italics and bold***".into();
        let result = text_parse(&text);
        assert!(result.is_ok());
        let inner = Box::into_inner(result.unwrap());
        assert!(inner == BlogBody::Bold(
            Box::new(BlogBody::Italics(
                Box::new(BlogBody::Text("italics and bold".into()))
            ))
        ))
    }
    
    #[test]
    fn test_parse_bold_with_inner_italics() {
        let text: String = "**bold*italics*bold**".into();
        let result = text_parse(&text);
        assert!(result.is_ok());
        let inner = Box::into_inner(result.unwrap());
        assert!(inner == BlogBody::Bold(
            Box::new(BlogBody::Span(Span{ elements:
                vec![ 
                    BlogBody::Text("bold".into()),
                    BlogBody::Italics(
                        Box::new(BlogBody::Text("italics".into()))
                    ),
                    BlogBody::Text("bold".into()),
                ]
            }))
        ))
    }

    #[test]
    fn test_parse_inline_note() {
        let text: String = "[^0]".into();
        let result = text_parse(&text);
        assert!(result.is_ok());
        let inner = Box::into_inner(result.unwrap());
        assert!(inner == BlogBody::Note("0".into()))
    }

    #[test]
    fn test_parse_citation_alphabetic() {
        let text: String = "[£example]".into();
        let result = text_parse(&text);
        assert!(result.is_ok());
        let inner = Box::into_inner(result.unwrap());
        assert!(inner == BlogBody::Citation("example".into()))
    }
    
    #[test]
    fn test_parse_citation_dash() {
        let text: String = "[£-other-example]".into();
        let result = text_parse(&text);
        assert!(result.is_ok());
        let inner = Box::into_inner(result.unwrap());
        assert!(inner == BlogBody::Citation("-other-example".into()))
    }
    
}


