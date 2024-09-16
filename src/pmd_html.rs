use color_print::cprintln;
use anyhow::anyhow;
use crate::*;

struct Reference<T> {
    def: T,
    times_used: usize
}

impl<T> Reference<T> {
    fn new(def: T) -> Self {
        Self { def, times_used: 0 }
    }
} 

pub struct PMDHTMLSerializer { 
    filename: String,
    toc: Option<TableOfContent>,
    references: HashMap<String, Reference<ReferenceDefinition>>,
    num_tabs:   usize,
    quote_id:   usize,
    list_id :   usize,
    code_id:    usize,
    image_id:   usize,
    missing_id: usize,
}

impl PMDHTMLSerializer {
    pub fn new(filename: &str) -> Self {
        Self { 
            filename: filename.into(),
            toc: None, 
            references: HashMap::new(),
            num_tabs:   0,
            quote_id:   0,
            list_id :   0,
            code_id:    0,
            image_id:   0,
            missing_id: 0,
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

    fn generate_missing_id(&mut self) -> String {
        let result = format!("missing-{}", self.missing_id); 
        self.missing_id += 1; 
        result
    }
    
    fn generate_missing_image_id(&mut self) -> String {
        let result = format!("image-{}", self.image_id); 
        self.image_id += 1; 
        result
    }
    
    fn generate_missing_code_id(&mut self) -> String {
        let result = format!("codeblock-{}", self.code_id); 
        self.code_id += 1; 
        result
    }
    
    fn generate_missing_list_id(&mut self) -> String {
        let result = format!("list-{}", self.list_id); 
        self.list_id += 1; 
        result
    }
    
    fn generate_missing_quote_id(&mut self) -> String {
        let result = format!("quote-{}", self.quote_id); 
        self.quote_id += 1; 
        result
    }

    fn prepare_header(&mut self, header: &BlogHeader, description: &String) -> String {
        let mut output = String::new();
        let title = &header.title;
        let banner = &header.banner;
        let url = &header.url;
        let data_dir = &header.data_dir;
        let blog_dir = &header.blog_dir;

        output += "<meta http-equiv=\"content-type\" content=\"text/html; charset=utf-8\">\n";
        output += "\n";
        output += "<!-- highlight.js -->\n";
        output += "<script src=\"https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.7.0/highlight.min.js\"></script>\n";
        output += "<link rel=\"stylesheet\" href=\"https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.7.0/styles/default.min.css\">\n";
        output += "<script>hljs.highlightAll();</script>\n";
        output += "\n";
        output += "<!-- tag needed for media query -->\n";
        output += "<meta name=\"viewport\"    content=\"width=device-width, initial-scale=1, minimum-scale=1\" />\n";
        output += "\n";
        output += "<!-- Primary Meta Tags -->\n";
        output += format!("<title>{}</title>\n", title).as_str();
        output += format!("<meta name=\"title\" content=\"{}\">\n", title).as_str();
        output += format!("<meta name=\"description\" content=\"{}\">\n", description).as_str();
        output += "\n";
        output += "<!-- Open Graph / Facebook -->\n";
        output += "<meta property=\"og:type\" content=\"website\">\n";
        output += format!("<meta property=\"og:url\" content=\"{url}/{blog_dir}/{}.html\">\n", self.filename).as_str();
        output += format!("<meta property=\"og:title\" content=\"{}\">\n", title).as_str();
        output += format!("<meta property=\"og:description\" content=\"{}\">\n", description).as_str();
        if banner.len() > 0 {
            output += format!("<meta property=\"og:image\" content=\"{url}/{blog_dir}/{}\">\n", banner).as_str();
        } else {
            output += format!("<meta property=\"og:image\" content=\"{url}/{data_dir}/minibanner.png\">\n").as_str();
        }
        output += "\n";
        output += "<!-- Twitter -->\n";
        if banner.len() > 0 {
            output += "<meta property=\"twitter:card\" content=\"summary_large_image\">\n";
        } else {
            output += "<meta property=\"twitter:card\" content=\"summary\">\n";
        }
        output += format!("<meta property=\"twitter:url\" content=\"{url}/{blog_dir}/{}.html\">\n", self.filename).as_str();
        output += format!("<meta property=\"twitter:title\" content=\"{}\">\n", title).as_str();
        output += format!("<meta property=\"twitter:description\" content=\"{}\">\n", description).as_str();
        if banner.len() > 0 {
            output += format!("<meta property=\"twitter:image\" content=\"{url}/{blog_dir}/{}\">\n", banner).as_str();
        } else {
            output += format!("<meta property=\"twitter:image\" content=\"{url}/{blog_dir}/minibanner.png\">\n").as_str();
        }
        output += "\n";
        output += "<!-- stylesheets -->\n";
        output += "<link rel=\"stylesheet\" href=\"../css/base.css\">\n";
        output += "<link rel=\"stylesheet\" href=\"../css/blog.css\">\n";
        output += "\n";
        output += "<!-- for Atkinson Hyperlegible font-->\n";
        output += "<link rel=\"preconnect\" href=\"https://fonts.googleapis.com\">\n";
        output += "<link rel=\"preconnect\" href=\"https://fonts.gstatic.com\" crossorigin>\n";
        output += "<link href=\"https://fonts.googleapis.com/css2?family=Atkinson+Hyperlegible:ital,wght@0,400;0,700;1,400;1,700&display=swap\" rel=\"stylesheet\">\n";
        output += "\n";
        output += "<script type='text/javascript' src='../js/popup.js'></script>\n";
        output += "<script type='text/javascript' src='../js/popup.js'></script>\n";
        output += "<script type='text/javascript' src='../js/dropdown.js'></script>\n";
        output += "<script type='text/javascript' src='../js/backref.js'></script>\n";
        
        output
    }

    fn generate_id<F: FnMut()->String>(text: &String, mut default_generator: F) -> String {
        if text.len() == 0 { default_generator() } else {

            let mut len = 0;
            for c in text.chars() {
                if c.is_ascii_punctuation() { 
                    break
                } else {
                    len += c.len_utf8();
                }
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

    fn element_link(&mut self, id: &String, opt_text: Option<&str>, opt_class: Option<&str>) -> String {
        let text  = opt_text.unwrap_or("¶");
        let class = opt_class.unwrap_or("paragraph");

        format!("<a class='{class}' href='#{id}' aria-hidden='true'>{text}</a>")
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

    result
}



impl PMDSerializer for PMDHTMLSerializer {
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

        let header = self.prepare_header(&md.header, &description);

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

        output += "<nav class=\"nav-bar\">\n";
        self.push_tab();
            output += self.tab().as_str();
            output += "<a class=\"paw-holder\" href=\"..\">\n";
            self.push_tab();
            output += self.tab().as_str();
            output += "<div class=\"paw-beans\">\n";
            self.push_tab();
                output += self.tab().as_str();
                output += "<div class=\"bean bean-nth-0\"></div>\n";
                output += self.tab().as_str();
                output += "<div class=\"bean bean-nth-1\"></div>\n";
                output += self.tab().as_str();
                output += "<div class=\"bean bean-nth-2\"></div>\n";
                output += self.tab().as_str();
                output += "<div class=\"bean bean-nth-3\"></div>\n";
            self.pop_tab();
            output += self.tab().as_str();
            output += "</div>\n";

            output += self.tab().as_str();
            output += "<div class=\"paw-pad\"></div>\n";

            self.pop_tab();
            output += self.tab().as_str();
            output += "</a>\n";
            
            output += self.tab().as_str();
            output += "<section>\n";
            self.push_tab();
                output += self.tab().as_str();
                output += "<button class='nav-phone-dropdown-button' onclick='toggle_dropdown()'>☰ </button>\n";
                output += self.tab().as_str();
                output += "<ul class='nav-list'>\n";
                self.push_tab();
                    output += self.tab().as_str();
                    output += "<li><a href='../about.html'>⭐ About</a></li>\n";
                    output += self.tab().as_str();
                    output += "<li><a href='../art.html'>🎨 Art</a></li>\n";
                    output += self.tab().as_str();
                    output += "<li><a href='../code.html'>🦄 Code</a></li>\n";
                    output += self.tab().as_str();
                    output += format!("<li><a href='md/{}.md'>📋 Raw</a></li>\n", self.filename).as_str();
                self.pop_tab();
                output += self.tab().as_str();
                output += "</ul>\n";
            self.pop_tab();
            output += self.tab().as_str();
            output += "</section>\n";

        self.pop_tab();
        output += self.tab().as_str();
        output += "</nav>\n";

        output += self.tab().as_str();
        output += "<ul id='phone-dropdown' class='nav-phone-dropdown off'>\n";
        self.push_tab();
            output += "<li><a href='../about.html'>⭐ About</a></li>\n";
            output += self.tab().as_str();
            output += "<li><a href='../art.html'>🎨 Art</a></li>\n";
            output += self.tab().as_str();
            output += "<li><a href='../code.html'>🦄 Code</a></li>\n";
            output += self.tab().as_str();
            output += format!("<li><a href='md/{}.md'>📋 Raw</a></li>\n", self.filename).as_str();
        self.pop_tab();
        output += self.tab().as_str();
        output += "</ul>\n";

        output += self.tab().as_str();
        output += "<main>\n";
        self.push_tab();

        {
            let title    = &md.header.title;
            let subtitle = &md.header.subtitle;
            output += self.tab().as_str();
            output += "<section class='title'>\n";
            self.push_tab();
                output += self.tab().as_str();
                output += format!("<h1>{title}</h1>\n").as_str();
                
                output += self.tab().as_str();
                output += format!("<p class='subtitle'>{subtitle}</p>\n").as_str();
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
            let id = PMDHTMLSerializer::generate_id(&md.header.notes_title, ||"missing".into());
            let title = &md.header.notes_title;
            let link  = self.element_link(&id, Some(format!("<h1>§</h1>").as_str()), Some("header"));

            
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
                output += link.as_str();
                output.push('\n');
                
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
                    output += "<sup>";
                    self.push_tab();
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
                    output += "</sup>";
                    output.push('\n');
            
                self.pop_tab();
                output += self.tab().as_str();
                output += "</section>\n";
            }
        }
        

        if md.references.len() != 0 {
            let id = PMDHTMLSerializer::generate_id(&md.header.bibliography_title, ||"missing".into());
            let title = &md.header.bibliography_title;
            let link  = self.element_link(&id, Some(format!("<h1>§</h1>").as_str()), Some("header"));

            output += self.tab().as_str();
            output += format!("<section class='page-break'>\n").as_str();
            self.push_tab();
                output += self.tab().as_str();
                output += "<hr>\n";
            self.pop_tab();
            output += self.tab().as_str();
            output += format!("</section>\n").as_str();

            output += self.tab().as_str();
            output += format!("<section id='{id}'>\n").as_str();
            self.push_tab();
                output += self.tab().as_str();
                output += link.as_str();
                output.push('\n');
                
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
                output += format!("<section class='citation' id='{key}'>\n").as_str();
                self.push_tab();
                    output += self.tab().as_str();
                    output += format!("<p>\n").as_str();
                    self.push_tab();
                    
                        output += to_html_bibliography(val).as_str();
                        output.push('\n');
                    
                        output += self.tab().as_str();
                        output += format!("<a href=''>").as_str();
                        output += "↩";
                        output += "</a>";
                        output.push('\n');
                    self.pop_tab();
                    output += self.tab().as_str();
                    output += format!("</p>\n").as_str();

                self.pop_tab();
                output += self.tab().as_str();
                output += "</section>\n";
            }
        }        

        self.pop_tab();
        output += self.tab().as_str();
        output +=   "</main>\n";


        output += self.tab().as_str();
        output +="<div id='popup' class='popup-hidden' aria-hidden='true'>\n";
        self.push_tab();

            output += self.tab().as_str();
            output +="<div class='popup-clickable-region' onclick='close_popup(this)' ></div>\n";
            output += self.tab().as_str();
            output +="<div class='popup-container'>\n";
            self.push_tab();

            output += self.tab().as_str();
            output +="<img id='popup-image'>\n";
            output += self.tab().as_str();
            output +="<p id='popup-caption'></p>\n";

        self.pop_tab();
        output += self.tab().as_str();
        output +="</div>\n";
        self.pop_tab();

        output += self.tab().as_str();
        output +="</div>\n";

        self.pop_tab();
        output += self.tab().as_str();
        output +=   "</body>\n";

        self.pop_tab();
        output += self.tab().as_str();
        output +=   "</html>\n";

        Ok(output)
    }

    fn convert_hoverable(&mut self, hoverable: &Alternative) -> Result<String> {
        let base = self.convert_element(&hoverable.base)?;
        let alt  = self.convert_element(&hoverable.alt)?;
        Ok(format!("<span class='hoverable'><span class='hover-base'>{base}</span><span class='hover-alt'>{alt}</span></span>"))
    }

    fn convert_styled(&mut self, styled: &Alternative) -> Result<String> {
        let text  = self.convert_element(&styled.alt)?;
        let style = self.convert_element(&styled.base)?;
        Ok(format!("<span class='embedded-style' style='{style}'>{text}</span>"))
    }
    
    // fn convert_embedded_link(&mut self, src: &String, alt: &String) -> Result<String> {
    // }

    fn convert_link(&mut self, link: &Alternative) -> Result<String> {
        let href = self.convert_element(&link.alt)?;
        let text = self.convert_element(&link.base)?;
        Ok(format!("<a class='inline-link' href='{href}'>{text}</a>"))
    }

    fn convert_header(&mut self, text: &Box<BlogBody>, depth: usize) -> Result<String> {
        let text  = self.convert_element(text)?;
        let id    = Self::generate_id(&text, ||"missing".into());
        let link  = self.element_link(&id, Some(format!("<h{depth}>§</h{depth}>").as_str()), Some("header"));
        
        let mut result = self.tab();
        result += format!("<section id='{id}'>\n").as_str();
        self.push_tab();
            result += self.tab().as_str();
            result += link.as_str();
            result.push('\n');
            
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
        let id = Self::generate_id(&"".into(), ||self.generate_missing_code_id());
        let link = self.element_link(&id, None, None);

        body = body.replace("&", "&amp;")
                   .replace("<", "&lt;")
                   .replace(">", "&gt;");
        body = body.trim_end().replace("\r\n", "\n");
        
        let mut result = self.tab();
        result += format!("<section class='code-block' id='{id}'>\n").as_str();
        self.push_tab();
            result += self.tab().as_str();
            result += format!("{link}\n").as_str();
            
            result += self.tab().as_str();
            result += format!("<pre><code class='language-{lang}'>{body}</code></pre>\n").as_str();
        self.pop_tab();
        result += self.tab().as_str();
        result += "</section>\n";

        Ok(result)
    }

    fn convert_image(&mut self, src: &String, alt: &String) -> Result<String> {
        let id = Self::generate_id(alt, || self.generate_missing_image_id());
        let link = self.element_link(&id, None, None);

        let mut result = self.tab();
        result += format!("<section class='image' id='{id}'>\n").as_str();
        self.push_tab();
            result += self.tab().as_str();
            result += format!("{link}\n").as_str();
            
            result += self.tab().as_str();
            result += format!("<img onclick='makePopup(this)' src='{src}' alt='{alt}'></img>\n").as_str();
        self.pop_tab();
        result += self.tab().as_str();
        result += "</section>\n";

        Ok(result)
    }

    fn convert_quote(&mut self, lines: &Vec<BlogBody>) -> Result<String> {
        let id = Self::generate_id(&"".to_string(), || self.generate_missing_quote_id());
        let link = self.element_link(&id, None, None);
        
        let mut quote_elements : Vec<String> = vec![];
        for elem in lines {
            let text = self.convert_element(elem)?;
            quote_elements.push(format!("{text}<br/>"));
        }
        let text = quote_elements.join("\n");

        let mut result = self.tab();
        result += format!("<section class='quote' id='{id}'>\n").as_str();
        self.push_tab();
            result += self.tab().as_str();
            result += format!("{link}\n").as_str();
            
            result += self.tab().as_str();
            result += format!("<div class='quote-line'></div><blockquote class='quote-text'>{text}</blockquote>\n").as_str();
        self.pop_tab();
        result += self.tab().as_str();
        result += "</section>\n";

        Ok(result)
    }

    fn convert_list(&mut self, list: &Vec<BlogBody>) -> Result<String> {
        let id = Self::generate_id(&"".to_string(), ||self.generate_missing_list_id());
        let link = self.element_link(&id, None, None);

        let mut result = self.tab();
        result += format!("<section class='list' id='{id}'>\n").as_str();
        self.push_tab();
            result += self.tab().as_str();
            result += format!("{link}\n").as_str();
            
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
        let id = Self::generate_id(&paragraph, || self.generate_missing_id());
        let link = self.element_link(&id, None, None);

        let mut result = self.tab();
        result += format!("<section id='{id}'>\n").as_str();
        self.push_tab();
            result += self.tab().as_str();
            result += format!("{link}\n").as_str();
            
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
            let num = reference.times_used;
            reference.times_used += 1;
            let text = to_citation(&reference.def);
            let mut result = String::new();
            result += "<cite>";
            result += format!("<a id='{id}-{num}' href='#{id}' onclick='backref(\"{id}\", \"{id}-{num}\")'>").as_str();
            result += text.as_str();
            result += "</a>";
            result += "</cite>";
            Ok(result)
        } else {
            cprintln!("<y>warning:</> {} has no source", id);
            Ok(format!("(MISSING CITATION)").to_string())
        }
    }
    

    fn convert_note(&mut self, id: &String) -> Result<String> {
        Ok(format!("<sup><a id='{id}-backref' href='#^{id}'>{id}</sup></a>"))
    }

    fn convert_toc(&mut self) -> Result<String> {
        if self.toc.is_none() { return Err(anyhow!("")); }
        let link  = self.element_link(&String::from("table-of-contents"), Some("<h1>§</h1>"), Some("header"));
        let toc = self.toc.clone().unwrap();
        let title = &toc.title;

        let mut result = self.tab();
        
        result += "<section id='table-of-contents'>\n";
        self.push_tab();
            result += self.tab().as_str();
            result += link.as_str();
            result.push('\n');
            
            result += self.tab().as_str();
            result += format!("<h1>{title}</h1>\n").as_str();
        self.pop_tab();
        result += self.tab().as_str();
        result += "</section>\n";

        result += self.tab().as_str();
        result += "<section>\n";
        self.push_tab();
            result += self.tab().as_str();
            result += "<ul>\n";
            self.push_tab();
                for (elem, depth) in &toc.headers {
                    let text = self.convert_element(elem)?;
                    let id    = Self::generate_id(&text, ||"missing".into());
                    result += self.tab().as_str();
                    result += format!("<li class='toci-{depth}'><a href='#{id}'>{text}</a></li>\n").as_str();
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
