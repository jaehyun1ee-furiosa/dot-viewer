use crate::app::{App, InputMode, Mode, SearchMode};
use crate::ui::ui::surrounding_block;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::Paragraph,
    Frame,
};

// input block
pub fn draw_input<B: Backend>(f: &mut Frame<B>, chunk: Rect, app: &mut App) {
    // surrounding block
    let title = match &app.mode {
        Mode::Navigate(_) => "Navigate",
        Mode::Input(input) => match input {
            InputMode::Search(search) => match search {
                SearchMode::Fuzzy => "Fuzzy Search",
                SearchMode::Regex => "Regex Search",
            },
            InputMode::Filter => "Filter",
        },
        _ => panic!("unreachable"),
    };

    let block = surrounding_block(
        title.to_string(),
        match app.mode {
            Mode::Input(_) => true,
            _ => false,
        },
    );

    f.render_widget(block, chunk);

    // inner blocks
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunk);
    draw_help(f, chunks[0], app);
    match app.mode {
        Mode::Navigate(_) => draw_error(f, chunks[1], app),
        Mode::Input(_) => draw_form(f, chunks[1], app),
        _ => panic!("unreachable"),
    };
}

// search block
pub fn draw_search<B: Backend>(f: &mut Frame<B>, chunk: Rect, app: &mut App) {
    // surrounding block
    let block = surrounding_block(
        "Search".to_string(),
        match app.mode {
            Mode::Input(_) => true,
            _ => false,
        },
    );
    f.render_widget(block, chunk);

    // inner blocks
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunk);
    draw_help(f, chunks[0], app);
    match app.mode {
        Mode::Navigate(_) => draw_error(f, chunks[1], app),
        Mode::Input(_) => draw_input(f, chunks[1], app),
        _ => panic!("unreachable"),
    };
}

// help block
fn draw_help<B: Backend>(f: &mut Frame<B>, chunk: Rect, app: &mut App) {
    let (msg, style) = match &app.mode {
        Mode::Navigate(_) => (
            vec![
                Span::raw("Press "),
                Span::styled(
                    "q",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" to exit, "),
                Span::styled(
                    "/",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" to fuzzy search, "),
                Span::styled(
                    "f",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" to apply filter with prefix, "),
                Span::styled(
                    "e",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" to export the current subgraph, "),
                Span::styled(
                    "0-9",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" to export the current neighbors with specified degree, "),
                Span::styled(
                    "c",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" to close tab, and "),
                Span::styled(
                    "tab/backtab",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" to change tab."),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        Mode::Input(input) => match input {
            InputMode::Search(_) => (
                vec![
                    Span::raw("Press "),
                    Span::styled(
                        "Esc",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" to stop search, "),
                    Span::styled(
                        "Enter",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" to goto selected node."),
                ],
                Style::default(),
            ),
            InputMode::Filter => (
                vec![
                    Span::raw("Press "),
                    Span::styled(
                        "Esc",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" to stop filter, "),
                    Span::styled(
                        "Enter",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" to apply filter."),
                ],
                Style::default(),
            ),
        },
        _ => panic!("unreachable"),
    };
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);

    let help = Paragraph::new(text);
    f.render_widget(help, chunk);
}

// error block
fn draw_error<B: Backend>(f: &mut Frame<B>, chunk: Rect, app: &mut App) {
    let msg = match &app.result {
        Ok(Some(msg)) => Some(msg.clone()),
        Err(err) => Some(format!("{}", err)),
        _ => None,
    };

    if let Some(msg) = msg {
        let msg =
            Paragraph::new(msg).style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
        f.render_widget(msg, chunk);
    }
}

// input block
fn draw_form<B: Backend>(f: &mut Frame<B>, chunk: Rect, app: &mut App) {
    let input = Paragraph::new(app.input.key()).style(match app.mode {
        Mode::Navigate(_) => Style::default(),
        Mode::Input(_) => Style::default().fg(Color::Yellow),
        _ => panic!("unreachable"),
    });
    f.render_widget(input, chunk);

    // cursor
    match app.mode {
        Mode::Navigate(_) => {}
        Mode::Input(_) => f.set_cursor(chunk.x + app.input.cursor as u16, chunk.y),
        _ => {},
    }
}
