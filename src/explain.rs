
pub fn explain(feature: &Option<String>) {
    let print_all = ||{
        println!("Arguments:");
        println!("    [Feature]");
        println!();
        println!("Features:");      
        println!("    title                 #[title] title text here");
        println!("    subtitle              #[subtitle] subtitle text here");
        println!("    banner                #[banner] path/to/banner");
        println!("    last-update           #[last-update] month day, year");
        println!("    notes-title           #[notes-title] New title");
        println!("    bibliography-title    #[bibliography-title] New title");
        println!("    toc                   #[toc] table of contents title here");
        println!("    pdf-header            #[pdf-header-left] text on the left");
        println!("    pdf-footer            #[pdf-footer-left] text on the left");
        println!("    image                 [[path/to/image] alt text here]");
        println!("    header                # header text here");
        println!("    codeblock             ```lang_name\\n text here ```");
        println!("    bold                  **bold text here**");
        println!("    italics               *italics text here*");
        println!("    hover                 %[base text](hovered text)");
        println!("    style                 £{{css_style: here;}}(text)");
        println!("    link                  [https://link.here](link text)");
        println!("    citation              [£some-citation]");
    };
    
    if let Some(feature) = feature {
        match feature.to_lowercase().as_str() {
            "title" | "titles"    => {
                println!("Titles:");
                println!("    a title is a special kind of header,");
                println!("    which is considered a level above all other headers.");
                println!();
                println!("Example: ");
                println!("    to write it, you use the symbol #[title]");
                println!();
                println!("    #[title] WoW what a title!");
                println!();
                println!("Note: ");
                println!("    the beginning whitespace between #[title] and the first");
                println!("    character will be removed");
            },
            "subtitle" | "subtitles" => {
                println!("Subtitles:");
                println!("    a subtitle is a special kind of header,");
                println!("    which is considered a level above all other headers");
                println!("    except for titles.");
                println!();
                println!("Example: ");
                println!("    to write it, you use the symbol #[subtitle]");
                println!();
                println!("    #[subtitle] WoW what a subtitle!");
                println!();
                println!("Note: ");
                println!("    the beginning whitespace between #[subtitle] and the first");
                println!("    character will be removed");
    
            },
            "banner" | "banners" => {
                println!("Banners:");
                println!("    banners are used when the page is embedded somewhere else,");
                println!("    like on twitter, or discord, #[banner] let's you specify ");
                println!("    where the picture for the banner is located");
                println!();
                println!("Example: ");
                println!("    #[banner] first-blog-images/banner.png");
                println!();
                println!("Note: ");
                println!("    the beginning whitespace between #[banner] and the first");
                println!("    character will be removed");
            },
            "image" | "images" => {
                println!("Images:");
                println!();
                println!("Example: ");
                println!("    [[path/to/image.png] an image, with alt text]");
                println!();
                println!("Note: ");
                println!();
            },
            "header" | "headers" => {
                println!("Headers:");
                println!();
                println!("Example: ");
                println!("    # a level 1 header");
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
                println!("    [https://link.goes.here](text goes here)");
                println!();
                println!("Note: ");
                println!();
            },
            "reference" | "references" | "citation" | "citations" => {
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
            },
            _ => print_all(),
        };
    } else {
        print_all();
    }
}
