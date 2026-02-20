//! Document viewing component with Markdown rendering

use iced::{
    Element, Length,
    widget::{self, Button, Column, Container, Scrollable, Space, Text},
};
use pulldown_cmark::{Parser, Options, html};

use crate::models::{Document, Page};
use crate::app::Message;

/// Document viewer state
#[derive(Debug, Clone, Default)]
pub struct State {
    /// Current document
    pub document: Option<Document>,
    /// Current page content
    pub current_page: Option<Page>,
    /// Rendered HTML (for future rich text support)
    pub rendered_content: String,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_document(document: Document) -> Self {
        let mut state = Self::new();
        state.document = Some(document.clone());
        // Use first page if available
        if let Some(page) = document.pages.first().cloned() {
            state.current_page = Some(page.clone());
            state.rendered_content = render_markdown(&page.content_markdown.unwrap_or_default());
        }
        state
    }
}

/// Render Markdown to HTML
pub fn render_markdown(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

/// Render the document view
pub fn view(state: &State) -> Element<'_, Message> {
    let mut content = Column::new();

    if let Some(doc) = &state.document {
        // Document header
        content = content
            .push(document_header(doc))
            .push(Space::with_height(Length::Fixed(16.0)));

        // Page navigation if multiple pages
        if doc.pages.len() > 1 {
            content = content
                .push(page_navigation(&doc.pages))
                .push(Space::with_height(Length::Fixed(16.0)));
        }

        // Page content
        if let Some(page) = &state.current_page {
            content = content.push(page_content(page));
        } else {
            content = content.push(
                Text::new("No content")
                    .size(14)
                    .color(iced::Color::from_rgb(0.6, 0.6, 0.6))
            );
        }
    } else {
        content = content.push(
            Text::new("Select a document to view")
                .size(16)
                .color(iced::Color::from_rgb(0.6, 0.6, 0.6))
        );
    }

    Container::new(Scrollable::new(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(24)
        .into()
}

fn document_header(doc: &Document) -> Element<'_, Message> {
    Column::new()
        .push(
            Text::new(&doc.name)
                .size(24)
        )
        .into()
}

fn page_navigation(pages: &[Page]) -> Element<'_, Message> {
    let mut row = widget::Row::new()
        .push(Text::new("Pages: ").size(12));

    for (i, page) in pages.iter().enumerate() {
        let btn = Button::new(
            Text::new(&page.name).size(12)
        )
        .padding([4, 8]);
        // Would add on_press here to switch pages
        
        row = row.push(btn);
        
        if i < pages.len() - 1 {
            row = row.push(Space::with_width(Length::Fixed(8.0)));
        }
    }

    row.into()
}

fn page_content(page: &Page) -> Element<'_, Message> {
    // For now, just show the raw markdown as text
    // In a future version, we could render HTML with a webview or rich text widget
    let text_content = page.content_markdown
        .as_ref()
        .or(page.content.as_ref())
        .map(|c| c.as_str())
        .unwrap_or("No content available");

    Column::new()
        .push(
            Text::new(&page.name)
                .size(18)
        )
        .push(Space::with_height(Length::Fixed(8.0)))
        .push(
            Scrollable::new(
                Text::new(text_content)
                    .size(14)
            )
            .height(Length::Fixed(400.0))
        )
        .into()
}
