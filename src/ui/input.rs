use crate::ui::surrounding_block;
use crate::viewer::{App, Mode, InputMode, SearchMode};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::Paragraph,
    Frame,
};

// input block
pub(super) fn draw_input<B: Backend>(
    f: &mut Frame<B>,
    chunk: Rect,
    app: &mut App,
) {
    // surrounding block
    let title = match &app.mode {
        Mode::Normal => "Normal",
        Mode::Input(imode) => match imode {
            InputMode::Search(smode) => match smode {
                SearchMode::Fuzzy => "Fuzzy Search",
                SearchMode::Regex => "Regex Search",
            }
            InputMode::Command => "Command",
        },
        _ => unreachable!(),
    };

    let block = surrounding_block(title.to_string(), matches!(app.mode, Mode::Input(_)));

    f.render_widget(block, chunk);

    // inner blocks
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunk);

    draw_error(f, chunks[0], app);
    draw_form(f, chunks[1], app);
}

// error block
fn draw_error<B: Backend>(f: &mut Frame<B>, chunk: Rect, app: &mut App) {
    let msg = match &app.result {
        Ok(succ) => succ.to_string(),
        Err(err) => err.to_string(),
    };

    if !msg.is_empty() {
        let msg =
            Paragraph::new(msg).style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
        f.render_widget(msg, chunk);
    }
}

// input block
fn draw_form<B: Backend>(f: &mut Frame<B>, chunk: Rect, app: &mut App) {
    let input = Paragraph::new(app.input.key.clone()).style(match &app.mode {
        Mode::Normal => Style::default(),
        Mode::Input(_) => Style::default().fg(Color::Yellow),
        _ => unreachable!(),
    });
    f.render_widget(input, chunk);

    // cursor
    match &app.mode {
        Mode::Normal => {}
        Mode::Input(_) => f.set_cursor(chunk.x + app.input.cursor as u16, chunk.y),
        _ => unreachable!(),
    }
}
