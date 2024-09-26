
use crate::*;
#[cfg(feature = "mock_pdf")]
use anyhow::anyhow;
#[cfg(all(feature = "pdf", not(feature = "mock_pdf")))]
use headless_chrome::Browser;

#[cfg(all(feature = "pdf", not(feature = "mock_pdf")))]
pub fn build_pdf(path: &str) -> Result<String> {
    let browser = Browser::default()?;
    let tab = browser.new_tab()?;
    
    tab.navigate_to(format!("file:{path}").as_str())?;
    
    tab.wait_until_navigated()?;
    
    Ok(unsafe { String::from_utf8_unchecked(tab.print_to_pdf(None)?) })
}

#[cfg(feature = "mock_pdf")]
pub fn build_pdf(_: &str) -> Result<String> {
    Err(anyhow!("this pdf was mocked, this is a message to the user to check the Cargo.toml, if this is intended as debug check the tmp.html file for the actual 'pdf'"))
}


