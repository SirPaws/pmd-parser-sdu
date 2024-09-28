# Paws Markdown parser (for SDU)
don't use this.

this is a fork of a scuffed markdown parser that I use for my blog,
this version specifically has some additions that I need for uni.

## PDF Frontmatter
for all other outputs frontmatter is ignored, but it's used for pdf files to add headers, footers, line height, and base text size 

```md
---
pdf-text-size: 11
pdf-line-height: 1
pdf-header-left: Something on the left!
pdf-footer-left: %p of %np
pdf-footer-right: %page of %pages
---
```
writing `pdf-footer` or `pdf-header` is equivalent to `pdf-footer-center`, and `pdf-header-center`
the `%p` and `%page` inserts the page number at the location (in the header), where `%np` and `%pages` inserts the number of pages


# How do I build it?
¯\\_(ツ)_/¯

