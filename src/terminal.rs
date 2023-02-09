use crate::{app::App, ui};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use std::io::Stdout;
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

fn setup() -> Result<Terminal<CrosstermBackend<Stdout>>, Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    Ok(terminal)
}

pub fn cleanup() -> Result<(), Box<dyn Error>> {
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    disable_raw_mode()?;

    Ok(())
}

pub fn launch(path: String) -> Result<(), Box<dyn Error>> {
    // setup terminal
    let mut terminal = setup()?;

    // run app
    let app = App::new(&path)?;
    let _ = run(&mut terminal, app);

    let _ = cleanup();

    Ok(())
}

fn run<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            app.key(key);
        }

        if app.quit {
            break;
        }
    } 

    Ok(())
}
