use color_print::cprintln;
use anyhow::anyhow;
use crate::*;
use tempfile::Builder;
use headless_chrome::Browser;
use std::io::Write;
use serde_yaml::Value;

struct Reference {
    def: ReferenceDefinition,
    times_used: usize
}

impl Reference {
    fn new(def: ReferenceDefinition) -> Self {
        Self { def, times_used: 0 }
    }
} 

pub struct PMDPDFSerializer { 
    #[allow(dead_code)]
    filename: String,
    toc: Option<TableOfContent>,
    references: HashMap<String, Reference>,
    num_tabs:   usize,
}

impl PMDPDFSerializer {
    pub fn new(filename: &str) -> Self {
        Self { 
            filename: filename.into(),
            toc: None,
            references: HashMap::new(),
            num_tabs:   0,
        }
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

    fn parse_pdf_string(text: &Value) -> String {
        if let Some(text) = text.as_string() {
            let mut text = text.clone();
            text.insert(0, '"');
            text.push('"');
            text = text.replace("%pages", "\"counter(pages)\"");
            text = text.replace("%page", "\"counter(page)\"");
            text = text.replace("%np", "\"counter(pages)\"");
            text = text.replace("%p",  "\"counter(page)\"");
            if text.len() == 0 { text = "none".to_string(); }
            text
        } else { "none".to_string() }
    }

    fn prepare_header(&mut self, frontmatter: &Option<Frontmatter>, max_depth: usize, title: &String, description: &String) -> String {
        let mut output = String::new();

        let frontmatter = frontmatter.clone().unwrap_or(Frontmatter::new());

        let top_left   = Self::parse_pdf_string(&frontmatter["pdf-header-left"]  );
        let mut top_center = Self::parse_pdf_string(&frontmatter["pdf-header-center"]);
        let top_right  = Self::parse_pdf_string(&frontmatter["pdf-header-left"]  );

        let bottom_left   = Self::parse_pdf_string(&frontmatter["pdf-footer-left"]  );
        let mut bottom_center = Self::parse_pdf_string(&frontmatter["pdf-footer-center"]);
        let bottom_right  = Self::parse_pdf_string(&frontmatter["pdf-footer-left"]  );

        if top_center == "" {
            top_center = Self::parse_pdf_string(&frontmatter["pdf-header"]);
        }
        if bottom_center == "" {
            bottom_center = Self::parse_pdf_string(&frontmatter["pdf-footer"]);
        }

        let text_size   = frontmatter["pdf-text-size"].as_i64();
        let line_height = frontmatter["pdf-line-height"].as_i64();

        output += "<meta http-equiv=\"content-type\" content=\"text/html; charset=utf-8\">\n";
        output += "\n";
        output += "<!-- tag needed for media query -->\n";
        output += "<meta name=\"viewport\"    content=\"width=device-width, initial-scale=1, minimum-scale=1\" />\n";
        output += "\n";
        output += "<!-- Primary Meta Tags -->\n";
        output += format!("<title>{title}</title>\n").as_str();
        output += format!("<meta name=\"title\" content=\"{title}\">\n").as_str();
        output += format!("<meta name=\"description\" content=\"{description}\">\n").as_str();
        output += "\n";
        output += "<!-- stylesheets -->\n";
        output += "<style>\n";
        output += "main {\n";
        output += "    font-family: 'Atkinson Hyperlegible', sans-serif;\n";
        if let Some(text_size) = text_size {
            output += format!("    font-size: {text_size}pt;\n").as_str();
        }
        if let Some(line_height) = line_height {
            output += format!("    line-height: {line_height};\n").as_str();
        }
        output += "}\n";
        output += "\n";
        output += "h1 {margin-top: 2em; margin-bottom: 0}\n";
        output += "h2 {margin-top: 2em; margin-bottom: 0}\n";
        output += "h3 {margin-top: 2em; margin-bottom: 0}\n";
        output += "h4 {margin-top: 2em; margin-bottom: 0}\n";
        output += "h5 {margin-top: 2em; margin-bottom: 0}\n";
        output += "h6 {margin-top: 2em; margin-bottom: 0}\n";
        output += ".bibliography h1 {margin-top: auto; margin-bottom: auto}\n";
        output += ".notes h1 {margin-top: auto; margin-bottom: auto}\n";
        output += "\n";
        output += ".banner { \n";
        output += "    width: 100%;\n";
        output += "}\n";
        output += "\n";
        output += ".image { \n";
        output += "    justify-content: center;\n";
        output += "    display: flex;\n";
        output += "}\n";
        output += "\n";
        output += ".quote { \n";
        output += "    justify-content: center;\n";
        output += "    margin-bottom: 1em;\n";
        output += "    display: flex;\n";
        output += "}\n";
        output += "\n";
        output += ".quote-line {\n";
        output += "    width: 1ch;\n";
        output += "    padding-left: 0;\n";
        output += "    padding-right: 1ch;\n";
        output += "    border-left: 0.3ch solid black;\n";
        output += "}\n";
        output += "\n";
        output += ".quote-text {\n";
        output += "    margin-left: 0;\n";
        output += "    margin-top: 0;\n";
        output += "    margin-bottom: 0;\n";
        output += "}\n";
        output += "\n";
        output += "a {\n";
        output += "    text-decoration: none;\n";
        output += "    color: black;\n";
        output += "}\n";
        output += "\n";
        output += ".inline-link:link {color: blue}\n";
        output += ".inline-link:visited {color: purple}\n";
        output += ".inline-link:active {color: red}\n";
        output += "\n";
        output += ".toc ul {\n";
        output += "    padding: 0;\n";
        output += "    list-style: none;\n";
        output += "    font-size: larger;\n";
        output += "}\n";
        output += "\n";
        output += ".toci-header {\n";
        output += "    position: relative;\n";
        output += "    overflow: hidden;\n";
        output += "}\n";
        output += "\n";
        output += ".toci-header::after {\n";
        output += "    position: absolute;\n";
        output += "    padding-left: .25ch;\n";
        output += "    content: \" . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . \"\n";
        output += "    \". . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . \"\n";
        output += "    \". . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . \";\n";
        output += "    text-align: right;\n";
        output += "}\n";
        output += "\n";
        output += ".toci-number {\n";
        output += "    min-width: 1ch;\n";
        output += "}\n";
        output += "\n";
        output += ".toci-number::after {\n";
        output += "    content: target-counter(attr(href), page);\n";
        output += "}\n";
        output += "\n";
        output += ".toci-1 {\n";
        output += "    display: grid;\n";
        output += "    grid-template-columns: auto max-content;\n";
        output += "    grid-template-areas: \"toci-header toci-number\";\n";
        output += "    align-items: end;\n";
        output += "    gap: 0.25rem;\n";
        output += "}\n";
        output += "\n";

        if max_depth > 1 {
            for i in 2..(max_depth + 1) {
                let margin = i - 1;
                output += format!(".toci-{i} {{\n").as_str();
                output += "    display: grid;\n";
                output += "    grid-template-columns: auto max-content;\n";
                output += "    grid-template-areas: \"toci-header toci-number\";\n";
                output += "    align-items: end;\n";
                output += "    gap: 0.25rem;\n";
                output += format!("    margin-left: {margin}em;\n").as_str();
                output += "}\n";
                output += "\n";
            }
        }
        

        output += "\n";
        output += "@media print {\n";
        output += "    .title {\n";
        output += "        page-break-after: always;\n";
        output += "    }\n";
        output += "\n";
        output += "    .toc {\n";
        output += "        page-break-after: always;\n";
        output += "    }\n";
        output += "\n";
        output += "    @page {\n";
        output += format!("       @top-left   {{content: {top_left  }}}\n").as_str();
        output += format!("       @top-center {{content: {top_center}}}\n").as_str();
        output += format!("       @top-right  {{content: {top_right }}}\n").as_str();
        output += "\n";
        output += format!("       @bottom-left   {{content: {bottom_left  }}}\n").as_str();
        output += format!("       @bottom-center {{content: {bottom_center}}}\n").as_str();
        output += format!("       @bottom-right  {{content: {bottom_right }}}\n").as_str();
        output += "    }\n";
        output += "\n";
        output += "   @page:first {\n";
        output += "       @top-left   {content: none}\n";
        output += "       @top-center {content: none}\n";
        output += "       @top-right  {content: none}\n";
        output += "\n";
        output += "       @bottom-left   {content: none}\n";
        output += "       @bottom-center {content: none}\n";
        output += "       @bottom-right  {content: none}\n";
        output += "   }\n";
        output += "}\n";
        output += "</style>\n";
        output += "\n";
        output += "<!-- for Atkinson Hyperlegible font-->\n";
        output += "<link rel=\"preconnect\" href=\"https://fonts.googleapis.com\">\n";
        output += "<link rel=\"preconnect\" href=\"https://fonts.gstatic.com\" crossorigin>\n";
        output += "<link href=\"https://fonts.googleapis.com/css2?family=Atkinson+Hyperlegible:ital,wght@0,400;0,700;1,400;1,700&display=swap\" rel=\"stylesheet\">\n";
        output += "\n";
        output += "<!-- paged.js -->\n";
        output += "<script src='https://unpkg.com/pagedjs/dist/paged.polyfill.js'></script>\n";
        output += "\n";
        output += "<!-- highlight.js -->\n";
        output += "<script src=\"https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.7.0/highlight.min.js\"></script>\n";
        output += "<link rel=\"stylesheet\" href=\"https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.7.0/styles/default.min.css\">\n";
        output += "<script>\n";
        output += "    class HighlightJSHandler extends Paged.Handler {\n";
        output += "        constructor(chunker, polisher, caller) {\n";
        output += "            super(chunker, polisher, caller);\n";
        output += "        }\n";
        output += "\n";
        output += "        afterParsed(parsed) {\n";
        output += "            hljs.highlightAll();\n";
        output += "            parsed.querySelectorAll('code').forEach((el) => {\n";
        output += "                hljs.highlightElement(el);\n";
        output += "            })\n";
        output += "        }\n";
        output += "    }\n";
        output += "\n";
        output += "    Paged.registerHandlers(HighlightJSHandler);\n";
        output += "</script>\n";
        output
    }

    fn generate_id<F: FnMut()->String>(text: &String, mut default_generator: F) -> String {
        if text.len() == 0 { default_generator() } else {

            let mut len = 0;
            for c in text.chars() {
                if c.is_ascii_punctuation() { 
                    break
                } else { len += c.len_utf8() }
            }
            if len == 0 || text[0..len].trim_start().trim_end().len() == 0 { default_generator() } else {
                let mut result = String::new();
                for c in text[0..len].trim_start().trim_end().chars() {
                    result.push(if c.is_whitespace() { '-' } else { c })
                }
                result.remove_matches(|x: char| x != '-' && x.is_ascii_punctuation());
                result.to_lowercase()
            }
        }
    }


}

pub fn to_html_bibliography(value: &ReferenceDefinition) -> String {
    
    let mut result = "".to_string();

    for (n, author) in value.authors.iter().enumerate() {
        let name = bibliograph_name(&author);
        result += name.trim();
        if n != value.authors.len() - 1 { result += ", " }
    }

    result += " (";
    let date = &value.date;
    if let Some(year) = date.get_year() {
        result += year.to_string().as_str();
    }
    
    if let Some(month) = date.get_month() {
        result.push(',');
        result.push(' ');
        result += month.to_string();
        result.push(' ');
    }

    if let Some(day) = date.get_day() {
        result += day.to_string().trim();
    }
    result.push(')');
    result.push('.');
    result.push(' ');

    result.push('"');
    result += value.title.trim();
    result.push('.');
    result.push('"');
    if !value.description.is_empty() {
        result.push(' ');
        result.push('[');
        result += value.description.trim();
        result.push(']');
    }

    if value.editors.len() != 0 {
        result += " edited by ";
        if value.editors.len() == 1 {
            let name = &value.editors[0];
            result += name.trim();
            result.push('.');
        } else {
            for (n, author) in value.editors.iter().enumerate() {
                let name = bibliograph_name(&author);
                result += name.trim();
                if n != value.editors.len() - 1 { result += ", " }
            }
        }
    }
    
    if value.translators.len() != 0 {
        result += ", translated by ";
        if value.translators.len() == 1 {
            let name = &value.translators[0];
            result += name.trim();
            result.push('.');
        } else {
            for (n, author) in value.translators.iter().enumerate() {
                let name = bibliograph_name(&author);
                result += name.trim();
                if n != value.translators.len() - 1 { result += ", " }
            }
        }
    }

    if !value.container_title.is_empty() {
        result += " in ";
        result += value.container_title.trim();
    }

    if any_non_empty(&[&value.volume, &value.issue, &value.pages, &value.edition, &value.version]) {
        result.push(' ');
        result.push('(');

        let mut found_one = false;
        if !value.version.is_empty() {
            result += value.version.trim();
            result += " vers.";
            found_one = true;
        }
        if !value.edition.is_empty() {
            if found_one {
                result += ", "
            }
            result += value.edition.trim();
            result += " ed.";
            found_one = true;
        }
        if !value.volume.is_empty() {
            if found_one {
                result += ", "
            }
            result += value.volume.trim();
            result += " vol.";
            found_one = true;
        }
        if !value.issue.is_empty() {
            if found_one {
                result += ", "
            }
            result += "issue ";
            result += value.issue.trim();
            found_one = true;
        }
        if !value.pages.is_empty() {
            if found_one {
                result += ", "
            }
            result += "pp. ";
            result += value.pages.trim();
        }
    
        result.push(')');
    }
    result.push('.');

    if !value.publisher.is_empty() {
        result.push(' ');
        result += value.publisher.trim();
        result.push('.');
    }

    let mut has_link = false;
    if value.link == value.doi && !value.link.is_empty() {
        let link = value.link.trim();
        result += ", ";
        result += format!("<a href={link}>{link}</a>").as_str();
        has_link = true;
    } else {
        if !value.doi.is_empty() {
            let doi = value.doi.trim();
            result.push(' ');
            if !value.link.is_empty() {
                result += "doi: ";
            }
            result += format!("<a href={doi}>{doi}</a>").as_str();
            has_link = true;
        }

        if !value.link.is_empty() {
            let link = value.link.trim();
            result += ", ";
            result += format!("<a href={link}>{link}</a>").as_str();
            has_link = true;
        }
    }

    if !value.esbn.is_empty() {
        if has_link {
            result += ", ";
        } else {
            result.push(' ');
        }

        result += "esbn: ";
        result += value.esbn.trim();
    }

    if value.date_retrieved.is_not_none() {
        result += " accessed ";
        if let Some(year) = date.get_year() {
            result += year.to_string().as_str();
        }
        
        if let Some(month) = date.get_month() {
            result.push(',');
            result.push(' ');
            result += month.to_string();
            result.push(' ');
            result += " ";
        }
        
        if let Some(day) = date.get_day() {
            result += day.to_string().as_str();
        }
    }

    result.replace("&", "&amp;")
}

impl PMDSerializer for PMDPDFSerializer {

    fn convert(&mut self, md: &PawsMarkdown) -> Result<String> {
        let description: String;
        let mut output = String::new();
        self.toc = md.header.toc.clone();

        for (id, reference) in &md.references {
            self.references.insert(id.clone(), Reference::new(reference.clone()));
        }

        let paragraph = md.body.iter().find(|&x| match x { BlogBody::Paragraph(_) => true, _ => false});
        if let Some(BlogBody::Paragraph(content)) = paragraph {
            let text = PMDPureTextSerializer::new().convert_paragraph(content)?;
            description = text.trim_end().to_string();
        } else {
            description = "".into();
        }

        let mut max_depth = 0;
        if let Some(toc) = md.header.toc.as_ref() {
            max_depth = toc.max_depth;
        }

        let header = self.prepare_header(&md.header.frontmatter, max_depth, &md.header.title, &description);

        output +=   "<!doctype html>\n";
        output +=   "<html>\n";
        self.push_tab();

        output += self.tab().as_str();
        output +=   "<head>\n";
        self.push_tab();
        for line in header.lines() {
            output += self.tab().as_str();
            output += line;
            output += "\n";
        }
        self.pop_tab();
        output += self.tab().as_str();
        output += "</head>\n";

        output += self.tab().as_str();
        output += "<body>\n";
        self.push_tab();

        output += self.tab().as_str();
        output += "<main>\n";
        self.push_tab();

        {
            let title    = &md.header.title;
            let subtitle = &md.header.subtitle;
            let banner   = &md.header.banner;
            output += self.tab().as_str();
            output += "<section class='title'>\n";
            self.push_tab();
                output += self.tab().as_str();
                output += format!("<h1>{title}</h1>\n").as_str();
                
                output += self.tab().as_str();
                output += format!("<p class='subtitle'>{subtitle}</p>\n").as_str();
                
                if banner.trim() != "" {
                    output += self.tab().as_str();
                    output += format!("<img src='{banner}'class='banner'></img>\n").as_str();
                }
            self.pop_tab();

            output += self.tab().as_str();
            output += format!("</section>\n").as_str();
        }

        for element in &md.body {
            let result = self.convert_element(element)?;
            match element {
                BlogBody::CodeBlock(_) => {
                    output += result.as_str();
                    output += "\n";
                },
                _ => {
                    output += result.as_str();
                }
            }
        }
        if md.notes.len() != 0 {
            let id = PMDPDFSerializer::generate_id(&md.header.notes_title, ||"missing".into());
            let title = &md.header.notes_title;
            
            output += self.tab().as_str();
            output += format!("<section class='page-break'>\n").as_str();
            self.push_tab();
                output += self.tab().as_str();
                output += "<hr>\n";
            self.pop_tab();
            output += self.tab().as_str();
            output += format!("</section>\n").as_str();
                
            output += self.tab().as_str();
            output += format!("<section class='notes' id='{id}'>\n").as_str();
            self.push_tab();
                output += self.tab().as_str();
                output += format!("<h1>{title}</h1>\n").as_str();
            self.pop_tab();
            output += self.tab().as_str();
            output += format!("</section>\n").as_str();

            for (key, val) in &md.notes {
                let result = self.convert_element(val)?;
                let link = format!("<a href='#^{key}'>^{key}:</a>");

                output += self.tab().as_str();
                output += format!("<section class='note' id=\"^{key}\">").as_str();
                output.push('\n');
                self.push_tab();

                    output += self.tab().as_str();
                    output += "<p>\n";

                    self.push_tab();
                        output += self.tab().as_str();
                        output += "<sup>\n";

                        self.push_tab();
                            output += self.tab().as_str();
                            output += link.as_str();
                            output.push('\n');
                            
                            output += self.tab().as_str();
                            output += result.as_str();
                            output.push('\n');
                            
                            output += self.tab().as_str();
                            output += format!("<a href=\"#{key}-backref\">").as_str();
                            output += "↩";
                            output += "</a>";
                            output.push('\n');
                        self.pop_tab();

                        output += self.tab().as_str();
                        output += "</sup>\n";
                    self.pop_tab();

                    output += self.tab().as_str();
                    output += "</p>";
                    output.push('\n');

                self.pop_tab();
                output += self.tab().as_str();
                output += "</section>\n";
            }
        }

        if md.references.len() != 0 {
            let id = PMDPDFSerializer::generate_id(&md.header.bibliography_title, ||"missing".into());
            let title = &md.header.bibliography_title;

            output += self.tab().as_str();
            output += format!("<section class='page-break'>\n").as_str();
            self.push_tab();
                output += self.tab().as_str();
                output += "<hr>\n";
            self.pop_tab();
            output += self.tab().as_str();
            output += format!("</section>\n").as_str();

            output += self.tab().as_str();
            output += format!("<section class='bibliography' id='{id}'>\n").as_str();
            self.push_tab();
                output += self.tab().as_str();
                output += format!("<h1>{title}</h1>\n").as_str();
            self.pop_tab();
            output += self.tab().as_str();
            output += format!("</section>\n").as_str();

            for (key, val) in &self.references {
                if val.times_used == 0 {
                    cprintln!("<y>warning:</> reference '{}' is not used and will not be included", key);
                }
            }
            
            for (key, val) in &md.references {
                let reference = self.references.get(key).unwrap();
                if reference.times_used == 0 { continue; }

                output += self.tab().as_str();
                output += format!("<section id='{key}'>\n").as_str();
                self.push_tab();
                    output += self.tab().as_str();
                    output += format!("<p>\n").as_str();
                    self.push_tab();
                    
                        output += to_html_bibliography(val).as_str();
                        output.push('\n');
                    
                    self.pop_tab();
                    output += self.tab().as_str();
                    output += "</p>\n";

                self.pop_tab();
                output += self.tab().as_str();
                output += "</section>\n";
            }
        }

        self.pop_tab();
        output += self.tab().as_str();
        output +=   "</main>\n";

        self.pop_tab();
        output += self.tab().as_str();
        output +=   "</body>\n";

        self.pop_tab();
        output += self.tab().as_str();
        output +=   "</html>\n";

        let mut html = Builder::new().suffix(".html").tempfile_in(".")?;

        writeln!(html, "{}", output)?;
        let path = html.path().to_str().expect("could not convert parth to &str");
        std::fs::write("tmp.html", output)?;

        let browser = Browser::default()?;
        let tab = browser.new_tab()?;

        tab.navigate_to(format!("file:{path}").as_str())?;
        
        tab.wait_until_navigated()?;
        
        Ok(unsafe { String::from_utf8_unchecked(tab.print_to_pdf(None)?) })
    }

    fn convert_hoverable(&mut self, hoverable: &Alternative) -> Result<String> {
        Ok(self.convert_element(&hoverable.base)?)
    }

    fn convert_styled(&mut self, styled: &Alternative) -> Result<String> {
        let text  = self.convert_element(&styled.alt)?;
        let style = self.convert_element(&styled.base)?;
        Ok(format!("<span class='embedded-style' style='{style}'>{text}</span>"))
    }

    fn convert_link(&mut self, link: &Alternative) -> Result<String> {
        let href = self.convert_element(&link.alt)?;
        let text = self.convert_element(&link.base)?;
        Ok(format!("<a class='inline-link' href='{href}'>{text}</a>"))
    }

    fn convert_header(&mut self, text: &Box<BlogBody>, depth: usize) -> Result<String> {
        let text  = self.convert_element(text)?;
        let id    = Self::generate_id(&text, ||"missing".into());

        let mut result = self.tab();
        result += format!("<section id={id}>\n").as_str();
        self.push_tab();
            result += self.tab().as_str();
            result += format!("<h{depth}>{text}</h{depth}>\n").as_str();
        self.pop_tab();
        result += self.tab().as_str();
        result += "</section>\n";

        Ok(result)
    }

    fn convert_italics(&mut self, text: &Box<BlogBody>) -> Result<String> {
        let inner_text = self.convert_element(&text)?;
        Ok(format!("<i>{inner_text}</i>"))
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
        let lang = words.nth(0).unwrap_or("plaintext");
        let mut body = text[text.find('\n').context("expected at least one line in codeblock")? + 1..].to_string();

        body = body.replace("&", "&amp;")
                   .replace("<", "&lt;")
                   .replace(">", "&gt;");
        body = body.trim_end().replace("\r\n", "\n");
        
        let mut result = self.tab();
        result += format!("<section class='code-block'>\n").as_str();
        self.push_tab();
            result += self.tab().as_str();
            result += format!("<pre><code class='language-{lang}'>{body}</code></pre>\n").as_str();
        self.pop_tab();
        result += self.tab().as_str();
        result += "</section>\n";

        Ok(result)
    }

    fn convert_image(&mut self, src: &String, alt: &String) -> Result<String> {
        let mut result = self.tab();
        result += format!("<section class='image'>\n").as_str();
        self.push_tab();
            result += self.tab().as_str();
            result += format!("<img src='{src}' alt='{alt}'></img>\n").as_str();
        self.pop_tab();
        result += self.tab().as_str();
        result += "</section>\n";

        Ok(result)
    }

    fn convert_quote(&mut self, lines: &Vec<BlogBody>) -> Result<String> {
        let mut quote_elements : Vec<String> = vec![];
        for elem in lines {
            let text = self.convert_element(elem)?;
            quote_elements.push(format!("{text}<br/>"));
        }
        let text = quote_elements.join("\n");

        let mut result = self.tab();
        result += format!("<section class='quote'>\n").as_str();
        self.push_tab();
            result += self.tab().as_str();
            result += format!("<div class='quote-line'></div><blockquote class='quote-text'>{text}</blockquote>\n").as_str();
        self.pop_tab();
        result += self.tab().as_str();
        result += "</section>\n";

        Ok(result)
    }

    fn convert_list(&mut self, list: &Vec<BlogBody>) -> Result<String> {
        let mut result = self.tab();
        result += format!("<section class='list'>\n").as_str();
        self.push_tab();
            result += self.tab().as_str();
            result += "<ul>";
            self.push_tab();
            for elem in list {
                let text = self.convert_element(elem)?;
                result += self.tab().as_str();
                result += format!("<li>{text}</li>\n").as_str();
            }
            self.pop_tab();
            result += self.tab().as_str();
            result += "</ul>";
        self.pop_tab();
        result += self.tab().as_str();
        result += "</section>\n";

        Ok(result)
    }

    fn convert_paragraph(&mut self, text: &Box<BlogBody>) -> Result<String> {
        let paragraph = self.convert_element(text)?;

        let mut result = self.tab();
        result += format!("<section>\n").as_str();
        self.push_tab();
            result += self.tab().as_str();
            result += "<p>\n";
            self.push_tab();
            
            for line in paragraph.trim_end().lines() {
                result += self.tab().as_str();
                result += line;
                result.push('\n');
            }

            self.pop_tab();
            result += self.tab().as_str();
            result += "</p>\n";
        self.pop_tab();
        result += self.tab().as_str();
        result += "</section>\n";

        Ok(result)
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
                self.convert_element(elem)?
            }.as_str()
        }
        Ok(result)
    }
    
    fn convert_citation(&mut self, id: &String) -> Result<String> {
        if let Some(reference) = self.references.get_mut(id) {
            reference.times_used += 1;
            let text = to_citation(&reference.def);
            let mut result = format!("<a href='#{id}'>").to_string();
            result += text.as_str();
            result += "</a>";
            Ok(result)
        } else {
            cprintln!("<y>warning:</> {} has no source", id);
            Ok(format!("(MISSING CITATION)").to_string())
        }
    }
    fn convert_note(&mut self, id: &String) -> Result<String> {
        Ok(format!("<sup><a id='{id}-backref' href='#^{id}'>{id}</a></sup>"))
    }

    fn convert_toc(&mut self) -> Result<String> {
        if self.toc.is_none() { return Err(anyhow!("")); }
        let toc = self.toc.clone().unwrap();
        let title = &toc.title;

        let mut result = self.tab();
        
        result += "<section id='table-of-contents'>\n";
        self.push_tab();
            result += self.tab().as_str();
            result += format!("<h1>{title}</h1>\n").as_str();
        self.pop_tab();
        result += self.tab().as_str();
        result += "</section>\n";

        result += self.tab().as_str();
        result += "<section class='toc'>\n";
        self.push_tab();
            result += self.tab().as_str();
            result += "<ul>\n";
            self.push_tab();
                for (elem, depth) in &toc.headers {
                    let text = self.convert_element(elem)?;
                    let id    = Self::generate_id(&text, ||"missing".into());
                    result += self.tab().as_str();
                    result += format!("<li class='toci-{depth}'>\n").as_str();
                    self.push_tab();
                        result += self.tab().as_str();
                        result += format!("<a class='toci-header' href='#{id}'>{text}</a>\n").as_str();
                        result += self.tab().as_str();
                        result += format!("<a class='toci-number' href='#{id}'></a>\n").as_str();
                    self.pop_tab();
                    result += self.tab().as_str();
                    result += "</li>\n";
                }
            self.pop_tab();
            result += self.tab().as_str();
            result += "</ul>\n";

        self.pop_tab();
        result += self.tab().as_str();
        result += "</section>\n";

        Ok(result)
    }
}
