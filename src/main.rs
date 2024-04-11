#![feature(let_chains)]
#![feature(box_into_inner)]
#![feature(string_remove_matches)]
#![feature(box_patterns)]
use std::{collections::HashMap, fs};
use anyhow::{Context, Result};

mod frontmatter;
mod references;
mod explain;
mod toplevel;
mod paws_markdown;
mod pmd_serializer;
mod pmd_pure_text;
#[cfg(feature = "html")]
mod pmd_html;
#[cfg(feature = "rss")]
mod pmd_rss;
#[cfg(feature = "pdf")]
mod pmd_pdf;

use frontmatter::*;
use references::*;
use toplevel::*;
use paws_markdown::*;
use pmd_serializer::*;
use pmd_pure_text::*;
#[cfg(feature = "html")]
use pmd_html::*;
#[cfg(feature = "rss")]
use pmd_rss::*;
#[cfg(feature = "pdf")]
use pmd_pdf::*;


// single feature interfaces ///////////////////////////////////////////////////////////////////
#[cfg(all(feature = "html", not(feature = "rss"), not(feature = "text"), not(feature = "pdf")))]
mod single_feature_html;
#[cfg(all(feature = "html", not(feature = "rss"), not(feature = "text"), not(feature = "pdf")))]
use single_feature_html::execute;

#[cfg(all(not(feature = "html"), feature = "rss", not(feature = "text"), not(feature = "pdf")))]
mod single_feature_rss;
#[cfg(all(not(feature = "html"), feature = "rss", not(feature = "text"), not(feature = "pdf")))]
use single_feature_rss::execute;

#[cfg(all(not(feature = "html"), not(feature = "rss"), feature = "text", not(feature = "pdf")))]
mod single_feature_text;
#[cfg(all(not(feature = "html"), not(feature = "rss"), feature = "text", not(feature = "pdf")))]
use single_feature_text::execute;

#[cfg(all(not(feature = "html"), not(feature = "rss"), not(feature = "text"), feature = "pdf"))]
mod single_feature_pdf;
#[cfg(all(not(feature = "html"), not(feature = "rss"), not(feature = "text"), feature = "pdf"))]
use single_feature_pdf::execute;

// dual feature interfaces ////////////////////////////////////////////////////////////////////
#[cfg(all(feature = "html", feature = "rss", not(feature = "text"), not(feature = "pdf")))]
mod dual_feature_html_rss;
#[cfg(all(feature = "html", feature = "rss", not(feature = "text"), not(feature = "pdf")))]
use dual_feature_html_rss::execute;

#[cfg(all(feature = "html", not(feature = "rss"), feature = "text", not(feature = "pdf")))]
mod dual_feature_html_text;
#[cfg(all(feature = "html", not(feature = "rss"), feature = "text", not(feature = "pdf")))]
use dual_feature_html_text::execute;

#[cfg(all(feature = "html", not(feature = "rss"), not(feature = "text"), feature = "pdf"))]
mod dual_feature_html_pdf;
#[cfg(all(feature = "html", not(feature = "rss"), not(feature = "text"), feature = "pdf"))]
use dual_feature_html_pdf::execute;

#[cfg(all(not(feature = "html"), feature = "rss", not(feature = "text"), feature = "pdf"))]
mod dual_feature_pdf_rss;
#[cfg(all(not(feature = "html"), feature = "rss", not(feature = "text"), feature = "pdf"))]
use dual_feature_pdf_rss::execute;

#[cfg(all(not(feature = "html"), not(feature = "rss"), feature = "text", feature = "pdf"))]
mod dual_feature_pdf_text;
#[cfg(all(not(feature = "html"), not(feature = "rss"), feature = "text", feature = "pdf"))]
use dual_feature_pdf_text::execute;

#[cfg(all(not(feature = "html"), feature = "rss", feature = "text", not(feature = "pdf")))]
mod dual_feature_rss_text;
#[cfg(all(not(feature = "html"), feature = "rss", feature = "text", not(feature = "pdf")))]
use dual_feature_rss_text::execute;

// triple feature interfaces //////////////////////////////////////////////////////////////////
#[cfg(all(feature = "html", feature = "rss", feature = "text", not(feature = "pdf")))]
mod triple_feature_html_rss_text;
#[cfg(all(feature = "html", feature = "rss", feature = "text", not(feature = "pdf")))]
use triple_feature_html_rss_text::execute;

#[cfg(all(feature = "html", feature = "rss", not(feature = "text"),feature = "pdf"))]
mod triple_feature_html_rss_pdf;
#[cfg(all(feature = "html", feature = "rss", not(feature = "text"),feature = "pdf"))]
use triple_feature_html_rss_pdf::execute;

#[cfg(all(feature = "html", not(feature = "rss"), feature = "text", feature = "pdf"))]
mod triple_feature_html_text_pdf;
#[cfg(all(feature = "html", not(feature = "rss"), feature = "text", feature = "pdf"))]
use triple_feature_html_text_pdf::execute;

#[cfg(all(not(feature = "html"), feature = "rss", feature = "text", feature = "pdf"))]
mod triple_feature_pdf_rss_text;
#[cfg(all(not(feature = "html"), feature = "rss", feature = "text", feature = "pdf"))]
use triple_feature_pdf_rss_text::execute;


// all features interface ///////////////////////////////////////////////////////////////////
#[cfg(all(feature = "html", feature = "rss", feature = "text", feature = "pdf"))]
mod all_features;
#[cfg(all(feature = "html", feature = "rss", feature = "text", feature = "pdf"))]
use all_features::execute;

/////////////////////////////////////////////////////////////////////////////////////////////

fn main() -> Result<()> {
    execute()
}
