
use crate::*;

pub trait PMDSerializer {
    fn convert_hoverable(&mut self, hoverable: &Alternative) -> Result<String>;
    fn convert_styled(&mut self, styled: &Alternative) -> Result<String>;
    fn convert_link(&mut self, link: &Alternative) -> Result<String>;
    fn convert_header(&mut self, text: &Box<BlogBody>, depth: usize) -> Result<String>;
    fn convert_italics(&mut self, text: &Box<BlogBody>) -> Result<String>;
    fn convert_bold(&mut self, text: &Box<BlogBody>) -> Result<String>;
    fn convert_inlinecode(&mut self, text: &String) -> Result<String>;
    fn convert_codeblock(&mut self, text: &String) -> Result<String>;
    fn convert_image(&mut self, src: &String, alt: &String) -> Result<String>;
    fn convert_quote(&mut self, lines: &Vec<BlogBody>) -> Result<String>;
    fn convert_list(&mut self, list: &Vec<BlogBody>) -> Result<String>;
    fn convert_paragraph(&mut self, text: &Box<BlogBody>) -> Result<String>;
    fn convert_text(&mut self, text: &String) -> Result<String>;
    fn convert_span(&mut self, span: &Span) -> Result<String>;
    fn convert_citation(&mut self, citation: &String) -> Result<String>;
    fn convert_note(&mut self, id: &String) -> Result<String>;
    fn convert_toc(&mut self) -> Result<String>;

    fn convert_element(&mut self, element: &BlogBody) -> Result<String> {
        match element {
            BlogBody::Hoverable(hoverable)    => self.convert_hoverable(hoverable),
            BlogBody::Styled(styled)          => self.convert_styled(styled),
            BlogBody::Link(link)              => self.convert_link(link),
            BlogBody::Header(text, depth)     => self.convert_header(text, *depth),
            BlogBody::Italics(text)           => self.convert_italics(text),
            BlogBody::Bold(text)              => self.convert_bold(text)   ,
            BlogBody::InlineCode(text)        => self.convert_inlinecode(text) ,
            BlogBody::CodeBlock(text)         => self.convert_codeblock(text)  ,
            BlogBody::Image(text, alt)        => self.convert_image(text, alt),
            BlogBody::Quote(lines)            => self.convert_quote(lines),
            BlogBody::List(list)              => self.convert_list(list),
            BlogBody::Paragraph(text)         => self.convert_paragraph(text),
            BlogBody::Text(text)              => self.convert_text(text),
            BlogBody::Span(span)              => self.convert_span(span),
            BlogBody::Citation(text)          => self.convert_citation(text),
            BlogBody::Note(text)              => self.convert_note(text),
            BlogBody::TOCLocationMarker       => self.convert_toc(),
        }
    }

    fn convert(&mut self, md: &PawsMarkdown) -> Result<String>;
}

pub fn to_string<T>(md: &PawsMarkdown, mut serialiser: T) -> Result<String>
    where T: PMDSerializer,
{
    serialiser.convert(md)
}
