use crate::*;

#[derive(Clone)]
struct Reference {
    def: ReferenceDefinition,
    times_used: usize,
}

impl Reference {
    fn new(def: ReferenceDefinition) -> Self {
        Self { def, times_used: 0 }
    }
} 

pub struct PMDRSSSerializer {
    filename: String,
    num_tabs:   usize,
    references: HashMap<String, Reference>,
    output: String,
}

impl PMDRSSSerializer {
    pub fn new(filename: &str) -> Self {
        Self { 
            filename: filename.into(),
            num_tabs: 0,
            references: HashMap::new(),
            output: String::new()
        }
    }

    fn push_line<S: AsRef<str>>(&mut self, text: S) {
        let result = self.tab();
        self.output += result.clone().as_str();
        self.output += text.as_ref();
        self.output.push('\n');
    }
    
    fn push_tab(&mut self) { self.num_tabs += 1; }
    fn pop_tab(&mut self) { if self.num_tabs > 0 { self.num_tabs -= 1; } }
    fn tab(&mut self) -> String {
        if self.num_tabs == 0 { "".to_string() }
        else {
            let mut result = String::new();
            for _ in 0..(self.num_tabs) {
                result += "    ";
            }
            result
        }
    } 
}

impl PMDSerializer for PMDRSSSerializer {
    fn convert_hoverable(&mut self, hoverable: &Alternative) -> Result<String> {
        let base = self.convert_element(&hoverable.base)?;
        let alt  = self.convert_element(&hoverable.alt)?;
        Ok(format!("<span><span>{base}</span><span>({alt})</span></span>"))
    }

    fn convert_styled(&mut self, styled: &Alternative) -> Result<String> {
        let text  = self.convert_element(&styled.alt)?;
        // let style = self.convert_element(&styled.base)?;
        Ok(format!("<span>{text}</span>"))
    }

    fn convert_link(&mut self, link: &Alternative) -> Result<String> {
        let href = self.convert_element(&link.alt)?;
        let text = self.convert_element(&link.base)?;
        Ok(format!("<a class='inline-link' href='{href}'>{text}</a>"))
    }

    fn convert_header(&mut self, text: &Box<BlogBody>, depth: usize) -> Result<String> {
        let text  = self.convert_element(text)?;
        Ok(format!("<h{depth}>{text}</h{depth}>"))
    }

    fn convert_italics(&mut self, text: &Box<BlogBody>) -> Result<String> {
        let inner_text = self.convert_element(&text)?;
        Ok(format!("<i>{inner_text}</i>)"))
    }

    fn convert_bold(&mut self, text: &Box<BlogBody>) -> Result<String> {
        let inner_text = self.convert_element(&text)?;
        Ok(format!("<b>{inner_text}</b>"))
    }

    fn convert_inlinecode(&mut self, text: &String) -> Result<String> {
        Ok(format!("<code>{text}</code>"))
    }

    fn convert_codeblock(&mut self, text: &String) -> Result<String> {
        let first_line = text.lines().nth(0).context("expected at least one line in codeblock")?;
        let mut words = first_line.split(|x: char| x.is_whitespace());
        let _lang = words.nth(0).unwrap_or("plaintext");
        let mut body = text[text.find('\n').context("expected at least one line in codeblock")? + 1..].to_string();

        body = body.replace("&", "&amp;")
                   .replace("<", "&lt;")
                   .replace(">", "&gt;");
        body = body.trim_end().replace("\r\n", "\n");

        Ok(format!("<pre><code>{body}</code></pre>"))
    }

    fn convert_image(&mut self, src: &String, alt: &String) -> Result<String> {
        Ok(format!("<img src='{src}' alt='{alt}'></img>"))
    }

    fn convert_quote(&mut self, lines: &Vec<BlogBody>) -> Result<String> {
        let mut quote_elements : Vec<String> = vec![];
        for elem in lines {
            let text = self.convert_element(elem)?;
            quote_elements.push(format!("{text}<br/>"));
        }
        let text = quote_elements.join("\n");
        Ok(format!("<blockquote class='quote-text'>{text}</blockquote>"))
    }

    fn convert_list(&mut self, list: &Vec<BlogBody>) -> Result<String> {
        let mut list_elements: Vec<String> = vec![];
        for elem in list {
            let text = self.convert_element(elem)?;
            list_elements.push(format!("<li>{text}</li>"));
        }
        let text = list_elements.join("\n");
        Ok(format!("<ul>{text}</ul>"))
    }

    fn convert_paragraph(&mut self, text: &Box<BlogBody>) -> Result<String> {
        let paragraph = self.convert_element(text)?;
        Ok(format!("<p>{paragraph}</p>"))
    }

    fn convert_text(&mut self, text: &String) -> Result<String> {
        Ok(text.trim_end().to_string())
    }

    fn convert_span(&mut self, span: &Span) -> Result<String> {
        let mut result = String::new();
        for elem in &span.elements {
            result += if let BlogBody::Text(text) = elem {
                text.to_string()
            } else {
                self.convert_element(elem)?
            }.as_str()
        }
        Ok(result)
    }
    
    fn convert_citation(&mut self, id: &String) -> Result<String> {
        if let Some(citation) = &self.references.get(id) {
            Ok(to_citation(&citation.def).trim_end().to_string())
        } else {
            Ok("(MISSING CITATION)".into())
        }
    }
    fn convert_note(&mut self, id: &String) -> Result<String> {
        Ok(format!("<sup><a id='{id}-backref' href='#^{id}'>{id}</sup></a>"))
    }

    fn convert_toc(&mut self) -> Result<String> {
        todo!()
    }

    fn convert(&mut self, md: &PawsMarkdown) -> Result<String> {
        let title = &md.header.title;

        for (key, val) in &md.references {
            self.references.insert(key.clone(), Reference::new(val.clone()));
        }

        let filename = self.filename.clone();
        let date = if md.header.last_update.is_not_none() {
                md.header.last_update.to_date().unwrap_or(chrono::Utc::now())
            } else {
                md.header.date_written.to_date().unwrap_or(chrono::Utc::now())
            }.to_rfc3339();

        self.push_line("<entry>\n");
        self.push_tab();
        self.push_line(format!("<title>{title}</title>"));
        self.push_line(format!("<link href=\"https://sirpaws.dev/blog/{filename}\"/>"));
        self.push_line(format!("<updated>{date}</updated>"));
        self.push_line(format!("<id>https://sirpaws.dev/blog/{filename}</id>"));
        self.push_line("<content type=\"xhtml\">");

        self.push_tab();
        for element in &md.body {
            let result = self.convert_element(element)?;
            self.push_line(result.trim_end());
        }

        if md.notes.len() != 0 {
            self.push_line("<hr>");
            for (key, val) in &md.notes {
                let result = self.convert_element(val)?;
                self.push_line("<p>");
                self.push_tab();
                        self.push_line(format!("<sup>^{key}:</sup>{result}"));
                self.pop_tab();
                self.push_line("</p>");
            }
        }


        if self.references.len() != 0 {
            self.push_line("<hr>");
            for (_, val) in self.references.clone() {
                if val.times_used == 0 { continue; }
                self.push_line("<p>");
                self.push_tab();
                    self.push_line(to_bibliography(&val.def));
                self.pop_tab();
                self.push_line("</p>");
            }
        }

        self.pop_tab();
        self.push_line("</content>");
        self.pop_tab();
        self.push_line("</entry>");
        
        Ok(self.output.clone())
    }    
}
