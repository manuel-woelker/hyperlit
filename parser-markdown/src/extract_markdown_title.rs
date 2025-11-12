use pulldown_cmark::{Event, Tag, TagEnd};

pub fn extract_markdown_title(markdown: &str) -> Option<String> {
    let parser = pulldown_cmark::Parser::new(markdown);
    let mut title = String::new();
    enum State {
        Init,
        InHeading,
        HasTitleText,
    }
    let mut state = State::Init;
    for event in parser {
        match event {
            Event::Start(Tag::Heading { .. }) => {
                state = State::InHeading;
            }
            Event::End(TagEnd::Heading { .. }) => {
                if let State::HasTitleText = state {
                    return Some(title);
                }
            }
            Event::Text(text) => {
                if let State::InHeading = state {
                    title.push_str(&text);
                    state = State::HasTitleText;
                }
            }
            _ => {
                // Ignore everything else
            }
        }
    }
    None
}
