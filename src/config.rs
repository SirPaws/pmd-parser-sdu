

pub const DEFAULT_URL: &'static str = "https://sirpaws.dev";
pub const DEFAULT_DATA_DIR: &'static str = "data";
pub const DEFAULT_BLOG_DIR: &'static str = "blog";
pub const DEFAULT_BIBLIOGRAPHY_TITLE: &'static str = "Litteraturliste";
pub const DEFAULT_NOTES_TITLE: &'static str = "Fodnoter";
pub const DEFAULT_CONTACT_TITLE: &'static str = "Kildeliste";
pub const DEFAULT_FACTBOX_TITLE: &'static str = "Fakta";

pub const MAX_ID_LENGTH: usize = 64;

pub const FRONTMATTER_HIDE_NOTES: [&'static str; 6] = [
    "hide-notes", "hide-endnotes", "hide-end-notes",
    "no-notes", "no-endnotes", "no-end-notes",
];

pub const FRONTMATTER_HIDE_REFERENCES: [&'static str; 9] = [
     "hide-bibliography", "hide-bib", "no-bib",
     "hide-references", "hide-refs", "no-refs",
     "hide-citations", "no-citations", "no-citation"
];

pub const FRONTMATTER_HIDE_CONTACTS: [&'static str; 2] = [
    "hide-contacts", "no-contacts"
];

pub const FRONTMATTER_SHOULD_CITE_CONTACTS: [&'static str; 4] = [
    "cite-contacts", "should-cite-contacts", "contacts-in-text",
    "contact-citation",
];
