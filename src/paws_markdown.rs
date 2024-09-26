use std::collections::HashSet;

#[cfg(not(feature = "wasm"))]
use color_print::cprintln;
use config::*;
use ordered_map::OrderedMap;
use crate::*;

macro_rules! no_id {
    ($e: expr) => { ($e, &String::new()) }
}

#[derive(Debug, PartialEq, Clone)]
pub struct TableOfContent {
    pub title:   String,
    pub index:   usize,
    pub max_depth: usize,
    pub headers: Vec<(Box<BlogBody>, /*depth: */ usize, /*id: */ String)>,
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
    pub hide_references: bool,
    pub hide_notes: bool,
    pub hide_contacts: bool,
    pub toc: Option<TableOfContent>,
    pub bibliography_title: String,
    pub notes_title: String,
    pub frontmatter: Option<Frontmatter>,
}

impl BlogHeader {
    pub fn default() -> Self {
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
            hide_references: false,
            hide_notes: false,
            hide_contacts: false,
            bibliography_title: DEFAULT_BIBLIOGRAPHY_TITLE.into(),
            notes_title: DEFAULT_NOTES_TITLE.into(),
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
    FactBox(FactBox),
    Quote(Vec<BlogBody>),
    List(Vec<BlogBody>),
    Paragraph(Box<BlogBody>),
    Text(String),
    Span(Span),
    Citation(String),
    Note(String),
    PageBreak,
    TOCLocationMarker,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PawsMarkdown {
    pub header: BlogHeader,
    pub bibliography_id: String,
    pub notes_id: String,
    pub references: OrderedMap<String, ReferenceDefinition>,
    pub notes: OrderedMap<String, BlogBody>,
    pub body: Vec<(BlogBody, String)>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FactBox {
    pub title: String,
    pub notes: OrderedMap::<String, (BlogBody, String)>,
    pub body: Vec<(BlogBody, String)>
}

fn gather_link<'l>(mut end: std::iter::Peekable<std::str::Chars<'l>>, depth: &mut i32) -> Result<(String, std::iter::Peekable<std::str::Chars<'l>>)> {
    let mut alt = String::new();
    if end.peek() == Some(&'(') {
        end.next();
        while !(end.peek() == Some(&')') && depth == &0) {
            let character = end.next().context("expected ')'")?;
            match character {
                '(' => *depth += 1,
                ')' => if depth != &0 { *depth -= 1 } else {},
                _   => {}
            }
            alt.push(character);
        }
        end.next();
        Ok((alt, end.clone()))
    }
    else {
        Ok((alt, end.clone()))
    }
}

fn get_citation(text: &String) -> Option<BlogBody> {
    if text.starts_with('£') && text.trim_start().chars().nth(1).is_some_and(|x| x.is_alphabetic() || x == '-') {
        // this is a citation
        let citation : String = text.chars().skip(1).collect();
        Some(BlogBody::Citation(citation))
    } else if text.starts_with('^') && text.len() > 1 {
        // this is a citation
        let citation : String = text.chars().skip(1).collect();
        Some(BlogBody::Note(citation))
    } else {
        None
    }
}

pub fn text_parse(text: &String) -> Result<(Box<BlogBody>, String)> {

    let mut body = Vec::<BlogBody>::new();
    let mut buffer = String::new();
    let mut peekable = text.chars().peekable();
    let mut tmp_id = String::new();
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
                            tmp_id.push(escaped_character);
                            if copy.peek().is_some_and(|&possible_brace| possible_brace == '[') {
                                peekable.next();
                                buffer.push('[');
                                tmp_id.push(escaped_character);
                            }
                        },
                        _ => {
                            buffer.push(escaped_character);
                            tmp_id.push(escaped_character);
                        }
                    }
                } else {
                    buffer.push('\\');
                    tmp_id.push('\\');
                }
            },
            '£'|'%' => {
                if buffer.len() != 0 {
                    body.push(BlogBody::Text(buffer));
                    buffer = String::new();
                }

                let start_char = *character;
                let mut make_object = |base: &String, alt: &String|
                    anyhow::Ok(if start_char == '%' { 
                        let (base, id) = text_parse(base)?;
                        let (alt, _) = text_parse(alt)?;
                        tmp_id += id.as_str();
                        BlogBody::Hoverable(Alternative{base, alt})
                    } else {
                        let (base, _) = text_parse(base)?;
                        let (alt,  id) = text_parse(alt)?;
                        tmp_id.push(' ');
                        tmp_id += id.as_str();
                        tmp_id.push(' ');
                        BlogBody::Styled(Alternative{base, alt})
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
                    tmp_id.push(start_char);
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
                let alt;
                (alt, peekable) = gather_link(end, &mut depth)?;
                if let Some(element) = get_citation(&alt) {
                    let (base, _) = text_parse(&base)?;
                    body.push(BlogBody::Link(
                        Alternative { base, alt: Box::new(element) }
                    ))
                } else {
                    let (base, id) = text_parse(&base)?;
                    let (alt, _) = text_parse(&alt)?;
                    tmp_id.push(' ');
                    tmp_id += id.as_str();
                    tmp_id.push(' ');

                    body.push(BlogBody::Link(Alternative{ base, alt}))
                }
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

                tmp_id.push(' ');
                tmp_id += base.as_str();
                tmp_id.push(' ');
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
                    let (text, id) = text_parse(&result)?;
                    tmp_id += id.as_str();
                    body.push(BlogBody::Bold(text))
                } else {
                    let mut result = String::new();
                    while peekable.peek() != Some(&'*') {
                        if peekable.peek().is_none() { break }
                        result.push(peekable.next().unwrap());
                    }
                    if peekable.peek() == Some(&'*') {
                        peekable.next();
                    }
                    let (text, id) = text_parse(&result)?;
                    tmp_id += id.as_str();
                    body.push(BlogBody::Italics(text))
                }
                continue;
                
            },
            _   => {
                buffer.push(*character); 
                tmp_id.push(*character); 
            },
        }

        peekable.next();
    }

    if buffer.len() != 0 {
        body.push(BlogBody::Text(buffer));
    }

    let id = generate_id(&tmp_id);

    match body.len() {
        0 => Ok((Box::new(BlogBody::Span(Span{elements: vec![]})), String::new())),
        1 => Ok((Box::new(body[0].clone()), id.unwrap_or(String::new()))),
        _ => Ok((Box::new(BlogBody::Span(Span{elements: body})), id.unwrap_or(String::new()))),
    }
}

fn generate_id(text: &String) -> Option<String> {
    if text.split_whitespace().collect::<String>().is_empty() {
        None
    } else {
        let words = text.split_whitespace().collect::<Vec<_>>();
        let mut id = String::new();
        for word in words {
            if id.len() + word.len() > MAX_ID_LENGTH {
                break;
            }
            id += word;
            id.push('-');
        }
        id.remove(id.len() - 1);
        Some(id)
    }
}

fn is_valid_id(text: &String) -> bool {
    !text.split_whitespace().collect::<String>().is_empty()
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

fn check_frontmatter(fm: &Frontmatter, keys: &[&str]) -> bool {
    for checked_key in keys {
        for key in fm.keys() {
            let key = key.trim().replace('_', "-").replace(' ', "-");
            if &key == checked_key {
                return true;
            }
        }
    }
    false
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

fn parse_factbox(toplevel_syntax: &Vec<TopLevelSyntax>) -> Result<PawsMarkdown> {
    let mut notes      = OrderedMap::<String, BlogBody>::new();
    let mut references = OrderedMap::<String, ReferenceDefinition>::new();
    let header: BlogHeader = BlogHeader::default();
    let mut body = Vec::<(BlogBody, String)>::new();
    
    let mut ids = HashSet::<String>::new();
    let mut num_codeblocks = 0usize;
    let mut num_image = 0usize;
    let mut num_lists = 0usize;
    let mut num_quotes = 0usize;

    for elem in toplevel_syntax {
        let last_length = body.len();
        match elem {
            TopLevelSyntax::FactBox{title: _, body: _} => {
                #[cfg(not(feature = "wasm"))]
                cprintln!("<r>error:</> fact boxes inside of fact boxes is not allowed");
            },
            TopLevelSyntax::FrontMatter(_) => { // (data)   => {
                #[cfg(not(feature = "wasm"))]
                cprintln!("<r>error:</> frontmatter inside of fact boxes is not allowed");
            }
            TopLevelSyntax::PageBreak => {
                body.push((BlogBody::PageBreak, String::new()));
            },
            TopLevelSyntax::CodeBlock(block) => { 
                body.push((
                        BlogBody::CodeBlock(block.to_string()),
                        if num_codeblocks == 0 {
                            format!("codeblock")
                        } else {
                            format!("codeblock-{num_codeblocks}")
                        }
                ));
                num_codeblocks = num_codeblocks + 1;
            },
            TopLevelSyntax::Image(img, alt) => {
                let id = generate_id(alt);
                body.push((
                        BlogBody::Image(img.to_string(), alt.to_string()),
                        id.unwrap_or(format!("image-{num_image}"))
                ));
                num_image = num_image + 1;
            },
            // TopLevelSyntax::EmbeddedLink(img, alt) => { body.push(BlogBody::EmbeddedLink(img.to_string(), alt.to_string())); },
            TopLevelSyntax::Header(text, level) => { 
                let (object, id) = text_parse(&text)?;
                body.push((BlogBody::Header(object, *level), id));
            },
            TopLevelSyntax::List(list) => {
                let mut result = Vec::<BlogBody>::new();
                for elem in list {
                    let (object, _) = text_parse(&elem)?;
                    result.push(Box::into_inner(object))
                }
                body.push((BlogBody::List(result), format!("list-{num_lists}")));
                num_lists = num_lists + 1;
            },
            TopLevelSyntax::Paragraph(text) => {
                let (object, id) = text_parse(&text)?;
                body.push((BlogBody::Paragraph(object), id));
            },
            TopLevelSyntax::Quote(list) => { 
                let mut result = Vec::<BlogBody>::new();
                for elem in list {
                    let (object, _) = text_parse(&elem)?;
                    result.push(Box::into_inner(object));
                }
                body.push((BlogBody::Quote(result), format!("quote-{num_quotes}")));
                num_quotes = num_quotes + 1;
            },
            TopLevelSyntax::ReferenceDefinition(reference) => {
                references.insert(reference.id.clone(), reference.clone());
            },
            TopLevelSyntax::NoteDefinition { id, text } => {
                let (object, _) = text_parse(&text)?;
                notes.insert(id.clone(), Box::into_inner(object));
            }
            TopLevelSyntax::TOC(_) => {
                #[cfg(not(feature = "wasm"))]
                cprintln!("<r>error:</> table of contents inside of fact boxes is not allowed");
            },
        };

        if body.len() != last_length {
            if let Some((_, id)) = body.last_mut() {
                if is_valid_id(id) {
                    while ids.contains(id) {
                        *id += format!("-{last_length}").as_str();
                    }
                    ids.insert(id.clone());
                }
            }
        }
    }

        Ok(PawsMarkdown { header, references, notes, body, notes_id: String::new(), bibliography_id: String::new() })
}

pub fn file_parse(file_path: &String) -> Result<PawsMarkdown> {
    parse(&fs::read_to_string(file_path)?, Some(file_path))
}

pub fn parse(file_content: &String, file_path: Option<&String>) -> Result<PawsMarkdown> {
    let toplevel_syntax = toplevel_parse(file_content)?;

    let mut notes      = OrderedMap::<String, BlogBody>::new();
    let mut references = OrderedMap::<String, ReferenceDefinition>::new();
    let mut header: BlogHeader = BlogHeader::default();
    let mut body = Vec::<(BlogBody, String)>::new();

    let mut ids = HashSet::<String>::new();
    let mut num_codeblocks = 0usize;
    let mut num_image = 0usize;
    let mut num_lists = 0usize;
    let mut num_quotes = 0usize;
    let mut num_factboxes = 0usize;

    for elem in &toplevel_syntax {
        let last_length = body.len();
        match elem {
            TopLevelSyntax::FactBox{ title, body: syntax} => {
                // Should return a Factbox object
                let factbox_parsed = parse_factbox(syntax)?;
                let mut factbox = FactBox {
                    title: title.clone(),
                    notes: OrderedMap::new(),
                    body: factbox_parsed.body,
                };

                if !factbox_parsed.references.is_empty() {
                    for (key, def) in &factbox_parsed.references {
                        references.insert(key.clone(), def.clone());
                    }
                }
                let id = if let Some(id) = generate_id(title) { id } else { format!("factbox-{num_factboxes}") };
                for (_, object_id) in &mut factbox.body {
                    if is_valid_id(object_id) {
                        *object_id = format!("{id}-{object_id}");
                        while ids.contains(object_id) {
                            *object_id += format!("-{last_length}").as_str();
                        }
                        ids.insert(id.clone());
                    }
                }

                for (key, elem) in &factbox_parsed.notes {
                    factbox.notes.insert(key.clone(), (elem.clone(), format!("{id}-{key}")))
                }

                body.push((
                        BlogBody::FactBox(factbox), 
                        id
                ));
                num_factboxes = num_factboxes + 1;
            },
            TopLevelSyntax::PageBreak => {
                body.push((BlogBody::PageBreak, String::new()));
            },
            TopLevelSyntax::FrontMatter(frontmatter) => {
                header.frontmatter = Some(frontmatter.clone());
            }
            TopLevelSyntax::CodeBlock(block) => {
                body.push((
                        BlogBody::CodeBlock(block.to_string()),
                        if num_codeblocks == 0 {
                            format!("codeblock")
                        } else {
                            format!("codeblock-{num_codeblocks}")
                        }
                ));
                num_codeblocks = num_codeblocks + 1;
            },
            TopLevelSyntax::Image(img, alt) => {
                let id = generate_id(alt);
                body.push((
                        BlogBody::Image(img.to_string(), alt.to_string()),
                        id.unwrap_or(format!("image-{num_image}"))
                ));
                num_image = num_image + 1;
            },
            // TopLevelSyntax::EmbeddedLink(img, alt) => { body.push(BlogBody::EmbeddedLink(img.to_string(), alt.to_string())); },
            TopLevelSyntax::Header(text, level) => {
                let (object, id) = text_parse(&text)?;
                body.push((BlogBody::Header(object, *level), id));
            },
            TopLevelSyntax::List(list) => {
                let mut result = Vec::<BlogBody>::new();
                for elem in list {
                    let (object, _) = text_parse(&elem)?;
                    result.push(Box::into_inner(object))
                }
                body.push((BlogBody::List(result), format!("list-{num_lists}")));
                num_lists = num_lists + 1;
            },
            TopLevelSyntax::Paragraph(text) => {
                let (object, id) = text_parse(&text)?;
                body.push((BlogBody::Paragraph(object), id));
            },
            TopLevelSyntax::Quote(list) => { 
                let mut result = Vec::<BlogBody>::new();
                for elem in list {
                    let (object, _) = text_parse(&elem)?;
                    result.push(Box::into_inner(object));
                }
                body.push((BlogBody::Quote(result), format!("quote-{num_quotes}")));
                num_quotes = num_quotes + 1;
            },
            TopLevelSyntax::ReferenceDefinition(reference) => {
                references.insert(reference.id.clone(), reference.clone());
            },
            TopLevelSyntax::NoteDefinition { id, text } => {
                let (object, _) = text_parse(&text)?;
                notes.insert(id.clone(), Box::into_inner(object));
            }
 
            TopLevelSyntax::TOC(title) => {
                if header.toc.is_none() {
                    header.toc = Some(TableOfContent{ title: title.clone(), index: body.len(), headers: vec![], max_depth: 1,});
                    body.push((BlogBody::TOCLocationMarker, String::new()));
                }
            },
        }

        if body.len() != last_length {
            if let Some((_, id)) = body.last_mut() {
                if is_valid_id(id) {
                    while ids.contains(id) {
                        *id += format!("-{last_length}").as_str();
                    }
                    ids.insert(id.clone());
                }
            }
        }
    }

    if let Some(frontmatter) = &header.frontmatter {
        if let Some(title) = frontmatter["title"].as_string() {
            header.title = title;
        } else {
            if let Some(file_path) = file_path {
                #[cfg(not(feature = "wasm"))]
                cprintln!("<r>error:</> Document '{}' is missing a title, see 'pmd explain frontmatter'", file_path);
            }
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
            if let Some(file_path) = file_path {
                #[cfg(not(feature = "wasm"))]
                cprintln!("<r>error:</> Document '{}' is missing a date, see 'pmd explain frontmatter'", file_path);
            }
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

        header.hide_notes      = check_frontmatter(frontmatter, &FRONTMATTER_HIDE_NOTES);
        header.hide_references = check_frontmatter(frontmatter, &FRONTMATTER_HIDE_REFERENCES);
        header.hide_contacts   = check_frontmatter(frontmatter, &FRONTMATTER_HIDE_CONTACTS);
    } else {
        //TODO(Paw): this should really be a warning
        if let Some(file_path) = file_path {
            #[cfg(not(feature = "wasm"))]
            cprintln!("<r>error:</> Document '{}' is missing frontmatter, see 'pmd explain frontmatter'", file_path);
        }
    }

    let notes_id     = if let Some(id) = generate_id(&header.notes_title) { id } else {
        let default_id = generate_id(&DEFAULT_NOTES_TITLE.to_string()).unwrap();
        default_id
    };
    let bibliography_id = if let Some(id) = generate_id(&header.bibliography_title) { id } else {
        let default_id = generate_id(&DEFAULT_BIBLIOGRAPHY_TITLE.to_string()).unwrap();
        default_id
    };

    if !notes.is_empty() {
        if ids.contains(&notes_id) {
            'outer: for (elem, id) in &mut body.iter_mut() {
                if let BlogBody::FactBox(factbox) = elem {
                    for (_, factbox_id) in &mut factbox.body {
                        if factbox_id != &notes_id { continue }
        
                        while ids.contains(factbox_id) {
                            *factbox_id = format!("{factbox_id}-disass");
                        }
                        break 'outer;
                    }
                }
                if id != &notes_id { continue }
                while ids.contains(id) {
                    *id = format!("{id}-disass");
                }
        
                break;
            }
        }
    }
    
    if !references.is_empty() {
        if ids.contains(&bibliography_id) {
            'outer: for (elem, id) in &mut body.iter_mut() {
                if let BlogBody::FactBox(factbox) = elem {
                    for (_, factbox_id) in &mut factbox.body {
                        if factbox_id != &bibliography_id { continue }
        
                        while ids.contains(factbox_id) {
                            *factbox_id = format!("{factbox_id}-disass");
                        }
                        break 'outer;
                    }
                }
                if id != &bibliography_id { continue }
                while ids.contains(id) {
                    *id = format!("{id}-disass");
                }
        
                break;
            }
        }
    }
    
    if let Some(toc) = header.toc.as_mut() {
        for (i, (item, id)) in body.iter().enumerate() {
            if i < toc.index { continue; }
            if let &BlogBody::Header(text, depth) = &item {
                if depth > &toc.max_depth {
                    toc.max_depth = *depth;
                }
                toc.headers.push((text.clone(), depth.clone(), id.clone()));
            }
            else if let &BlogBody::FactBox(_) = &item {
                if 2 > toc.max_depth {
                    toc.max_depth = 2;
                }
                toc.headers.push((Box::new(item.clone()), 2, id.clone()));
            }
        }

        if !notes.is_empty() {
            toc.headers.push((Box::new(BlogBody::Text(header.notes_title.clone())), 1, notes_id.clone()))
        }

        if !references.is_empty() {
            toc.headers.push((Box::new(BlogBody::Text(header.bibliography_title.clone())), 1, bibliography_id.clone()))
        }
    }

    Ok(PawsMarkdown { header, references, notes_id, bibliography_id, notes, body })
}


#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_parse_remove_escaped() {
        let text: String = "\\[]".into();
        let result = text_parse(&text);
        assert!(result.is_ok());
        let inner = Box::into_inner(result.unwrap().0);
        assert!(inner == BlogBody::Text("[]".into()));
    }
    
    #[test]
    fn test_parse_remove_escaped_embedding() {
        let text: String = "\\%[]".into();
        let result = text_parse(&text);
        assert!(result.is_ok());
        let inner = Box::into_inner(result.unwrap().0);
        assert!(inner == BlogBody::Text("%[]".into()));
    }
    
    #[test]
    fn test_parse_hover() {
        let text: String = "%[abc](def)".into();
        let result = text_parse(&text);
        assert!(result.is_ok());
        let inner = Box::into_inner(result.unwrap().0);
        assert!(inner == BlogBody::Hoverable(Alternative{ base: Box::new(BlogBody::Text("abc".into())), alt: Box::new(BlogBody::Text("def".into()))}));
    }
    
    #[test]
    fn test_parse_styling() {
        let text: String = "£{abc}(def)".into();
        let result = text_parse(&text);
        assert!(result.is_ok());
        let inner = Box::into_inner(result.unwrap().0);
        assert!(inner == BlogBody::Styled(Alternative{ base: Box::new(BlogBody::Text("abc".into())), alt: Box::new(BlogBody::Text("def".into()))}));
    }
    
    #[test]
    fn test_parse_hover_with_inner_styling_left() {
        let text: String = "%[£{style}(text)](alternative)".into();
        let result = text_parse(&text);
        assert!(result.is_ok());
        let inner = Box::into_inner(result.unwrap().0);
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
        let inner = Box::into_inner(result.unwrap().0);
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
        let inner = Box::into_inner(result.unwrap().0);
        assert!(inner == BlogBody::Link(Alternative{ base: Box::new(BlogBody::Text("abc".into())), alt: Box::new(BlogBody::Text("def".into()))}));
    }
    
    #[test]
    fn test_parse_link_with_styling() {
        let text: String = "[link](£{style}(text))".into();
        let result = text_parse(&text);
        assert!(result.is_ok());
        let inner = Box::into_inner(result.unwrap().0);
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
        let inner = Box::into_inner(result.unwrap().0);
        assert!(inner == BlogBody::InlineCode("here's some code".into()))
    }

    #[test]
    fn test_parse_italics() {
        let text: String = "*italics*".into();
        let result = text_parse(&text);
        assert!(result.is_ok());
        let inner = Box::into_inner(result.unwrap().0);
        assert!(inner == BlogBody::Italics(Box::new(BlogBody::Text("italics".into()))))
    }
    
    #[test]
    fn test_parse_bold() {
        let text: String = "**bold**".into();
        let result = text_parse(&text);
        assert!(result.is_ok());
        let inner = Box::into_inner(result.unwrap().0);
        assert!(inner == BlogBody::Bold(Box::new(BlogBody::Text("bold".into()))))
    }
    
    #[test]
    fn test_parse_bold_and_italics() {
        let text: String = "***italics and bold***".into();
        let result = text_parse(&text);
        assert!(result.is_ok());
        let inner = Box::into_inner(result.unwrap().0);
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
        let inner = Box::into_inner(result.unwrap().0);
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
        let inner = Box::into_inner(result.unwrap().0);
        assert!(inner == BlogBody::Note("0".into()))
    }

    #[test]
    fn test_parse_citation_alphabetic() {
        let text: String = "[£example]".into();
        let result = text_parse(&text);
        assert!(result.is_ok());
        let inner = Box::into_inner(result.unwrap().0);
        assert!(inner == BlogBody::Citation("example".into()))
    }
    
    #[test]
    fn test_parse_citation_dash() {
        let text: String = "[£-other-example]".into();
        let result = text_parse(&text);
        assert!(result.is_ok());
        let inner = Box::into_inner(result.unwrap().0);
        assert!(inner == BlogBody::Citation("-other-example".into()))
    }
    
}


