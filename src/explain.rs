
pub fn explain(feature: &Option<String>, extra: &Option<String>) {
    let print_all = ||{
        println!("Arguments:");
        println!("    [Feature]");
        println!();
        println!("Features:");      
        println!("    frontmatter           ---");
        println!("    title                 #[title] title text here");
        println!("    subtitle              #[subtitle] subtitle text here");
        println!("    banner                #[banner] path/to/banner");
        println!("    last-update           #[last-update] month day, year");
        println!("    notes-title           #[notes-title] New title");
        println!("    bibliography-title    #[bibliography-title] New title");
        println!("    toc                   #[toc] table of contents title here");
        println!("    image                 [[path/to/image] alt text here]");
        println!("    link                  [[http://example.link]]");
        println!("    header                # header text here");
        println!("    codeblock             ```lang_name\\n text here ```");
        println!("    bold                  **bold text here**");
        println!("    italics               *italics text here*");
        println!("    hover                 %[base text](hovered text)");
        println!("    style                 £{{css_style: here;}}(text)");
        println!("    link                  [link text](https://link.here)");
        println!("    citation              [£some-citation]");
    };
    
    if let Some(feature) = feature {
        match feature.to_lowercase().as_str() {
            "frontmatter" => {
                if let Some(_) = extra {
                    unimplemented!();
                    /*
                    match extra.to_lowercase().as_str() {
                        "title" => {},
                        "subtitle" => {},
                        "banner" => {},
                        "notes-title" => {},
                        "bibliography-title" => {},
                        "references-title" => {},
                        "sources-title" => {},
                        "bibliography title" => {},
                        "references title" => {},
                        "sources title" => {},
                        "bibliography_title" => {},
                        "references_title" => {},
                        "sources_title" => {},
                        "date" => {},
                        "date written" => {},
                        "date-written" => {},
                        "date_written" => {},
                        "last-update" => {},
                        "last_update" => {},
                        "last update" => {},
                        "last-updated" => {},
                        "last_updated" => {},
                        "last updated" => {},
                        "pdf-no-first-page" => {},
                        "pdf-text-size" => {},
                        "pdf-line-height" => {},
                        "pdf-font" => {},
                        "pdf-header" => {},
                        "pdf-header-left" => {},
                        "pdf-header-center" => {},
                        "pdf-header-right" => {},
                        "pdf-footer" => {},
                        "pdf-footer-left" => {},
                        "pdf-footer-center" => {},
                        "pdf-footer-right" => {},
                        _ => {}
                    }
                    */

                } else {
                    println!("Frontmatter:");
                    println!("    Frontmatter is a way to store meta information about the file");
                    println!("    currently this is only used for pdf files, but in the future");
                    println!("    more might be supported.");
                    println!();
                    println!("    As the name implies, frontmatter is always at the front");
                    println!("    (i.e. the top) of the file, and is formatted as YAML.");
                    println!();
                    println!("    the only items that will be looked for are");
                    println!("    - pdf-text-size");
                    println!("    - pdf-header-left");
                    println!("    - pdf-header-center");
                    println!("    - pdf-header-right");
                    println!("    - pdf-header");
                    println!("    - pdf-footer-left");
                    println!("    - pdf-footer-center");
                    println!("    - pdf-footer-right");
                    println!("    - pdf-footer");
                    println!("Example: ");
                    println!("    To make a frontmatter section you wrap it with three dashes");
                    println!();
                    println!("    ---");
                    println!("    item: this is frontmatter");
                    println!("    ---");
                    println!();
                    println!("Note: ");
                    println!("    the key's 'pdf-header' and 'pdf-footer' are equivalent to");
                    println!("    'pdf-header-center' and 'pdf-footer-center', having");
                    println!("    both will ignore the short versions");
                }
            },
            "title" | "titles"    => {
                println!("Titles:");
                println!("    A title is a special kind of header,");
                println!("    which is considered a level above all other headers.");
                println!();
                println!("Example: ");
                println!("    To write it, you use the symbol #[title]");
                println!();
                println!("    #[title] WoW what a title!");
                println!();
                println!("Note: ");
                println!("    The beginning whitespace between #[title] and the first");
                println!("    character will be removed");
            },
            "subtitle" | "subtitles" => {
                println!("Subtitles:");
                println!("    A subtitle is a special kind of header,");
                println!("    which is considered a level above all other headers");
                println!("    except for titles.");
                println!();
                println!("Example: ");
                println!("    To write it, you use the symbol #[subtitle]");
                println!();
                println!("    #[subtitle] WoW what a subtitle!");
                println!();
                println!("Note: ");
                println!("    The beginning whitespace between #[subtitle] and the first");
                println!("    character will be removed.");
                println!();
                println!("    Currently subtitles are used in rss for dates.");
                //TODO: dates should be part of frontmatter
    
            },
            "banner" | "banners" => {
                println!("Banners:"); //TODO: this should be part of the frontmatter
                println!("    Banners are used when the page is embedded somewhere else,");
                println!("    like on twitter, or discord, #[banner] let's you specify ");
                println!("    where the picture for the banner is located");
                println!();
                println!("Example: ");
                println!("    #[banner] first-blog-images/banner.png");
                println!();
                println!("Note: ");
                println!("    The beginning whitespace between #[banner] and the first");
                println!("    character will be removed");
            },
            "embedded" | "embedding" | "url" | "web" => {
                println!("Links:");
                println!();
                println!("Example: ");
                println!("    [[http://example.link] some alt text]");
                println!("    [[https://example.link] some alt text]");
                println!();
                println!("Note: ");
                println!();
            },
            "image" | "images" => {
                println!("Images:");
                println!();
                println!("Example: ");
                println!("    [[path/to/image.png] an image, with alt text]");
                println!();
                println!("Note: ");
                println!("    uses Open Graph or Twitter meta tags to get embedding");
                println!();
            },
            "header" | "headers" => {
                println!("Headers:");
                println!();
                println!("Example: ");
                println!("    #  A level 1 header");
                println!("    ## A level 2 header");
                println!();
                println!("Note: ");
                println!();
            },
            "codeblock" | "codeblocks" => {
                println!("Code Blocks:");
                println!();
                println!("Example: ");
                println!("    ```language");
                println!("    code here");
                println!("    ```");
                println!();
                println!("Note: ");
                println!();
            },
            "bold"      => {
                println!("Bold Text:");
                println!();
                println!("Example: ");
                println!("    **this text will be bold**");
                println!();
                println!("Note: ");
                println!();
            },
            "italics" | "italic"  => {
                println!("Italics Text:");
                println!();
                println!("Example: ");
                println!("    *this text will be italicised*");
                println!();
                println!("Note: ");
                println!();
            },
            "hover"     => {
                println!("Hover:");
                println!();
                println!("Example: ");
                println!("    %[this text will be changed](to this when you hover over it)");
                println!();
                println!("Note: ");
                println!("    on some devices like phones there is no hover,");
                println!("    but it will still change if you click on it");
                println!();
            },
            "style" | "styling" => {
                println!("Styling:");
                println!();
                println!("Example: ");
                println!("    £{{css-property: here;}}(text goes here)");
                println!();
                println!("Note: ");
                println!();
            },
            "link" | "links" => {
                println!("Links:");
                println!();
                println!("Example: ");
                println!("    [text goes here](https://link.goes.here)");
                println!();
                println!("Note: ");
                println!();
            },
            "reference" | "references" | "citation" | "citations" => {
                if let Some(_) = extra {
                    unimplemented!();
                    /*
                    match extra.to_lowercase().as_str() {
                        "title" => {},
                        "description" => {},
                        "container-title" => {},
                        "publisher" => {},
                        "edition" => {},
                        "version" => {},
                        "issue" => {},
                        "volume" => {},
                        "pages" => {},
                        "link" => {},
                        "doi" => {},
                        "esbn" => {},
                        "date" => {},
                        "day" => {},
                        "month" => {},
                        "year" => {},
                        "date-retrieved" => {},
                        "day-retrieved" => {},
                        "month-retrieved" => {},
                        "year-retrieved" => {},
                        "author" => {},
                        "authors" => {},
                        "editor" => {},
                        "editors" => {},
                        "translator" => {},
                        "translators" => {},
                        _ => {}
                    }
                    */
                } else {
                    println!("References:");
                    println!();
                    println!("Example: ");
                    println!("    £baudrillard {{");
                    println!("        title: Simulacra and Simulation,");
                    println!("        author: Jean Baudrillard,");
                    println!("        publisher: University of Michigan Press,");
                    println!("        year: 1994,");
                    println!("        pages: 176,");
                    println!("        esbn: 0-472-06521-1,");
                    println!("    }}");
                    println!("    ");
                    println!("    to actually reference these you type the name in a link");
                    println!("    [£baudrillard]");
                    println!();
                    println!("Note: ");
                    println!();
                }
            },
            _ => print_all(),
        };
    } else {
        print_all();
    }
}
