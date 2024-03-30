use crate::*;
use anyhow::*;

pub struct PMDPureTextSerializer {
    pub toc: Option<TableOfContent>,
    pub references: HashMap<String, ReferenceDefinition>,
}

impl PMDPureTextSerializer {
    pub fn new() -> Self { Self { toc: None, references: HashMap::new()} }
}

impl PMDSerializer for PMDPureTextSerializer {
    fn convert_hoverable(&mut self, hoverable: &Alternative) -> Result<String> {
        let alt    = self.convert_element(&hoverable.alt)?;
        let actual = self.convert_element(&hoverable.base)?;
        Ok(format!("{alt}({actual})"))
    }

    fn convert_styled(&mut self, styled: &Alternative) -> Result<String> {
        self.convert_element(&styled.alt)
    }

    fn convert_link(&mut self, link: &Alternative) -> Result<String> {
        let text = self.convert_element(&link.alt)?;
        let link = self.convert_element(&link.base)?;
        Ok(format!("{link}({text})"))
    }

    fn convert_header(&mut self, text: &Box<BlogBody>, _: usize) -> Result<String> {
        self.convert_element(text)
    }

    fn convert_italics(&mut self, text: &Box<BlogBody>) -> Result<String> {
        self.convert_element(text)
    }

    fn convert_bold(&mut self, text: &Box<BlogBody>) -> Result<String> {
        self.convert_element(text)
    }

    fn convert_inlinecode(&mut self, text: &String) -> Result<String> {
        Ok(text.clone())
    }

    fn convert_codeblock(&mut self, text: &String) -> Result<String> {
        let mut result :String = "-----\n".to_string();
        result += text.as_str();
        result += "-----\n";
        Ok(result)
    }

    fn convert_image(&mut self, src: &String, alt: &String) -> Result<String> {
        Ok(format!("{alt}({src})"))
    }

    fn convert_quote(&mut self, lines: &Vec<BlogBody>) -> Result<String> {
        let mut quote_elements : Vec<String> = vec![];
        for elem in lines {
            let text = self.convert_element(elem)?;
            quote_elements.push(format!("{text}\n"));
        }
        let text = quote_elements.join("\n");
        Ok(format!("\"{text}\""))
    }
    
    fn convert_citation(&mut self, citation: &String) -> Result<String> {
        let maybe_source = self.references.get(citation);
        if let Some(source) = maybe_source {
            Ok(to_citation(source))
        } else {
            Ok("(Missing Source)".to_string())
        }
    }
    
    fn convert_note(&mut self, id: &String) -> Result<String> {
        Ok(format!("^{id}").to_string())
    }

    fn convert_list(&mut self, list: &Vec<BlogBody>) -> Result<String> {
        let mut list_elements: Vec<String> = vec![];
        for elem in list {
            let text = self.convert_element(elem)?;
            list_elements.push(format!("- {text}"));
        }
        let text = list_elements.join("\n");
        Ok(text)
    }

    fn convert_paragraph(&mut self, text: &Box<BlogBody>) -> Result<String> {
        self.convert_element(text)
    }

    fn convert_text(&mut self, text: &String) -> Result<String> {
        Ok(text.clone())
    }

    fn convert_span(&mut self, span: &Span) -> Result<String> {
        let mut result = String::new();
        for elem in &span.elements {
            result += if let BlogBody::Text(text) = elem {
                text.to_string()
            } else {
                let mut inner = String::new();
                inner += self.convert_element(elem)?.as_str();
                inner
            }.as_str()
        }
        Ok(result)
    }
    
    fn convert_toc(&mut self) -> Result<String> {
        if self.toc.is_none() { return Err(anyhow!("")); }
        let mut result = String::new();
        let toc = self.toc.clone().unwrap();
        let title = &toc.title;

        result += format!("{title}:\n").as_str();
        for (text, depth) in &toc.headers {
            for _ in 0..depth.clone() {
                result += "    ";
            }
            result += self.convert_element(&text)?.as_str();
            result += "\n"
        }

        Ok(result)
    }

    fn convert(&mut self, md: &PawsMarkdown) -> Result<String> {
        let mut output = String::new();
        self.references = md.references.clone();
        self.toc = md.header.toc.clone();

        for element in &md.body {
            let result = self.convert_element(element)?;
            output += result.as_str();
            match element {
                BlogBody::Header(_, _) => {
                    output += ":\n";
                },
                _ => {
                    output += "\n";
                    output += "\n";
                }
            }
        }
        output += "--------------------------------------------------------------------------------\n";
        output += format!("{}: \n", md.header.notes_title).as_str();
        for (key, val) in &md.notes {
            let result = self.convert_element(val)?;
            output += format!("    ^{key}: {result}\n").as_str();
        }
        output.push('\n');

        output += "--------------------------------------------------------------------------------\n";
        output += format!("{}: \n", md.header.bibliography_title).as_str();
        output += "References: \n";
        for (_, val) in &self.references {
            output += to_bibliography(val).as_str();
            output += "\n";
        }

        Ok(output)
    }
}
