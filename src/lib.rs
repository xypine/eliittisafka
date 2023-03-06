use actix_web::rt::task::spawn_blocking;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const PAGE_URL: &str = "https://ksyk.fi";
const LANG_SEPARATOR: &str = "*";

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MenuInfo {
    pub title: String,
    pub content: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MenuDay {
    pub name: String,
    pub content: Vec<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Menu {
    pub info: Option<MenuInfo>,
    pub days: Vec<MenuDay>,
}

impl Menu {
    pub fn new_empty() -> Self {
        Self {
            info: None,
            days: vec![],
        }
    }
}

pub fn get_page_html() -> Result<String> {
    let response = reqwest::blocking::get(PAGE_URL)?;

    response.text().context("Failed to get page text content")
}

pub fn parse_html(raw: &str) -> scraper::Html {
    scraper::Html::parse_document(raw)
}

pub fn collect_text(
    parent: &scraper::ElementRef,
    min_length: usize,
    collapse_brackets: bool,
    language_separator_text: Option<&str>,
) -> Vec<Vec<String>> {
    let mut out: Vec<Vec<String>> = vec![];
    let mut current = vec![];

    for t in parent.text() {
        let trimmed = t.trim();
        if Some(trimmed) == language_separator_text {
            out.push(current);
            current = vec![];
        } else {
            if collapse_brackets
                && trimmed.starts_with("(")
                && trimmed.ends_with(")")
                && out.len() > 0
            {
                let mut last = current.pop().unwrap();
                last = format!("{} {}", last, trimmed);
                current.push(last);
            } else {
                if trimmed.len() >= min_length {
                    current.push(trimmed.to_string());
                }
            }
        }
    }
    out.push(current);

    out
}

pub fn extract_data(document: scraper::Html, extract_info: bool) -> Result<Menu> {
    let tab_data_container_selector =
        scraper::Selector::parse("div.et_pb_module.et_pb_tabs").unwrap();
    let tab_data_container = document
        .select(&tab_data_container_selector)
        .next()
        .context("could not find menu container")?;

    let menu_title_selector = scraper::Selector::parse(".et_pb_tabs_controls > *").unwrap();
    let mut menu_titles: Vec<scraper::ElementRef> =
        tab_data_container.select(&menu_title_selector).collect();

    let menu_item_selector = scraper::Selector::parse("div.et_pb_tab_content").unwrap();
    let mut menu_items: Vec<scraper::ElementRef> =
        tab_data_container.select(&menu_item_selector).collect();

    let mut menu = Menu::new_empty();

    if extract_info && menu_items.len() > 0 {
        let info_title = menu_titles.remove(0);
        let info_item = menu_items.remove(0);
        menu.info = Some(MenuInfo {
            title: collect_text(&info_title, 0, false, None)[0].join(""),
            content: collect_text(&info_item, 0, false, None)[0].clone(),
        });
    }

    let mut item_index = 0;
    for menu_item in menu_items {
        let day_name_element = menu_titles.get(item_index).unwrap();
        let name = collect_text(day_name_element, 0, false, None)[0].join("");
        let content = collect_text(&menu_item, 1, true, Some(LANG_SEPARATOR));
        menu.days.push(MenuDay { name, content });
        item_index += 1;
    }

    Ok(menu)
}

/// The main method you should be using
pub fn get_menu() -> Result<Menu> {
    println!("Downloading page...");
    let html = get_page_html().context("Failed to get page html")?;
    println!("Parsing html...");
    let document = parse_html(&html);
    println!("Extracting data...");
    let items = extract_data(document, true)?;
    println!("Menu extracted succesfully!");
    Ok(items)
}

/// For use in async contexts
pub async fn get_menu_async() -> Result<Menu> {
    spawn_blocking(move || get_menu())
        .await
        .expect("failed to await blocking task")
}
