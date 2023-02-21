use crate::viewer::{
    app::App,
    error::{DotViewerError, DotViewerResult},
    modes::{Mode, PopupMode, SearchMode},
    success::Success,
    view::{Focus, View},
};

use crossterm::event::{KeyCode, KeyEvent};
use log::{info, warn};

impl App {
    pub fn key(&mut self, key: KeyEvent) {
        info!("{:?}", key.code);

        self.result = match key.code {
            KeyCode::Char(c) => self.char(c).map(|_| Success::default()),
            KeyCode::Enter => self.enter(),
            KeyCode::Backspace => self.backspace().map(|_| Success::default()),
            KeyCode::Esc => self.esc().map(|_| Success::default()),
            KeyCode::Tab => self.tab().map(|_| Success::default()),
            KeyCode::BackTab => self.backtab().map(|_| Success::default()),
            _ => Ok(Success::default()),
        };

        if let Err(err) = &self.result {
            warn!("{}", err);
        }

        self.lookback = Some(key.code);
    }

    fn char(&mut self, c: char) -> DotViewerResult<()> {
        match &self.mode {
            Mode::Normal => self.char_normal(c),
            Mode::Command => self.char_command(c),
            Mode::Search(_) => self.char_search(c),
            Mode::Popup(_) => self.char_popup(c),
        }
    }

    fn char_normal(&mut self, c: char) -> DotViewerResult<()> {
        match c {
            'q' => self.quit = true,
            '/' => self.set_search_mode(SearchMode::Fuzzy),
            'r' => self.set_search_mode(SearchMode::Regex),
            ':' => self.set_command_mode(),
            'c' => self.tabs.close()?,
            'n' => {
                let view = self.tabs.selected();
                view.matches.next();
                view.goto_match()?
            }
            'N' => {
                let view = self.tabs.selected();
                view.matches.previous();
                view.goto_match()?
            }
            'h' => self.left()?,
            'j' => self.down()?,
            'k' => self.up()?,
            'l' => self.right()?,
            'g' => {
                if let Some(KeyCode::Char('g')) = self.lookback {
                    let view = self.tabs.selected();
                    view.first()?;
                }
            }
            'G' => {
                let view = self.tabs.selected();
                view.last()?;
            }
            _ => Err(DotViewerError::KeyError(KeyCode::Char(c)))?,
        };

        Ok(())
    }

    fn char_command(&mut self, c: char) -> DotViewerResult<()> {
        self.input.insert(c);
        Ok(())
    }

    fn char_search(&mut self, c: char) -> DotViewerResult<()> {
        self.input.insert(c);

        match &self.mode {
            Mode::Search(smode) => {
                let view = self.tabs.selected();
                let key = &self.input.key;

                match smode {
                    SearchMode::Fuzzy => view.update_fuzzy(key),
                    SearchMode::Regex => view.update_regex(key),
                }
                view.update_trie();

                view.goto_match()
            }
            _ => unreachable!(),
        }
    }

    fn char_popup(&mut self, c: char) -> DotViewerResult<()> {
        match &self.mode {
            Mode::Popup(pmode) => match pmode {
                PopupMode::Tree => self.char_tree(c),
                PopupMode::Help => self.char_help(c),
            },
            _ => unreachable!(),
        }
    }

    fn char_tree(&mut self, c: char) -> DotViewerResult<()> {
        match c {
            'q' => {
                self.quit = true;
                Ok(())
            }
            'h' => self.left(),
            'j' => self.down(),
            'k' => self.up(),
            'l' => self.right(),
            _ => Err(DotViewerError::KeyError(KeyCode::Char(c))),
        }
    }

    fn char_help(&mut self, c: char) -> DotViewerResult<()> {
        match c {
            'q' => {
                self.quit = true;
                Ok(())
            }
            'j' => self.down(),
            'k' => self.up(),
            _ => Err(DotViewerError::KeyError(KeyCode::Char(c))),
        }
    }

    fn enter(&mut self) -> DotViewerResult<Success> {
        match &self.mode {
            Mode::Normal => {
                let view = self.tabs.selected();
                view.enter().map(|_| Success::default())
            }
            Mode::Command => self.exec(),
            Mode::Search(_) => {
                self.set_normal_mode();
                Ok(Success::default())
            }
            Mode::Popup(pmode) => match pmode {
                PopupMode::Tree => self.subgraph().map(|_| Success::default()),
                _ => Ok(Success::default()),
            },
        }
    }

    fn backspace(&mut self) -> DotViewerResult<()> {
        match &self.mode {
            Mode::Command => self.input.delete(),
            Mode::Search(smode) => {
                self.input.delete();

                let view = self.tabs.selected();
                let key = &self.input.key;

                match smode {
                    SearchMode::Fuzzy => view.update_fuzzy(key),
                    SearchMode::Regex => view.update_regex(key),
                };
                view.update_trie()
            }
            _ => Err(DotViewerError::KeyError(KeyCode::Backspace))?,
        };

        Ok(())
    }

    fn esc(&mut self) -> DotViewerResult<()> {
        match &self.mode {
            Mode::Normal => Err(DotViewerError::KeyError(KeyCode::Esc)),
            _ => {
                self.set_normal_mode();
                Ok(())
            }
        }
    }

    fn tab(&mut self) -> DotViewerResult<()> {
        match &self.mode {
            Mode::Normal => self.tabs.next(),
            Mode::Command => self.autocomplete_command(),
            Mode::Search(smode) => match smode {
                SearchMode::Fuzzy => self.autocomplete_fuzzy(),
                SearchMode::Regex => self.autocomplete_regex(),
            },
            _ => Err(DotViewerError::KeyError(KeyCode::Tab))?,
        };

        Ok(())
    }

    fn backtab(&mut self) -> DotViewerResult<()> {
        match &self.mode {
            Mode::Normal => {
                self.tabs.previous();
                Ok(())
            }
            _ => Err(DotViewerError::KeyError(KeyCode::BackTab)),
        }
    }

    fn up(&mut self) -> DotViewerResult<()> {
        let view = self.tabs.selected();

        match &self.mode {
            Mode::Normal => view.up()?,
            Mode::Popup(pmode) => match pmode {
                PopupMode::Tree => view.subtree.up(),
                PopupMode::Help => self.help.previous(),
            },
            _ => Err(DotViewerError::KeyError(KeyCode::Up))?,
        };

        Ok(())
    }

    fn down(&mut self) -> DotViewerResult<()> {
        let view = self.tabs.selected();

        match &self.mode {
            Mode::Normal => view.down()?,
            Mode::Popup(pmode) => match pmode {
                PopupMode::Tree => view.subtree.down(),
                PopupMode::Help => self.help.next(),
            },
            _ => Err(DotViewerError::KeyError(KeyCode::Down))?,
        };

        Ok(())
    }

    fn right(&mut self) -> DotViewerResult<()> {
        match &self.mode {
            Mode::Normal => {
                let view = self.tabs.selected();
                view.right()
            }
            Mode::Search(_) => self.input.front(),
            Mode::Popup(PopupMode::Tree) => {
                let view = self.tabs.selected();
                view.subtree.right()
            }
            _ => Err(DotViewerError::KeyError(KeyCode::Right))?,
        };

        Ok(())
    }

    fn left(&mut self) -> DotViewerResult<()> {
        match &self.mode {
            Mode::Normal => {
                let view = self.tabs.selected();
                view.left()
            }
            Mode::Search(_) => self.input.back(),
            Mode::Popup(PopupMode::Tree) => {
                let view = self.tabs.selected();
                view.subtree.left()
            }
            _ => Err(DotViewerError::KeyError(KeyCode::Left))?,
        };

        Ok(())
    }
}

impl View {
    pub(super) fn enter(&mut self) -> DotViewerResult<()> {
        match &self.focus {
            Focus::Prev | Focus::Next => self.goto_adjacent(),
            Focus::Current => Ok(()),
        }
    }

    pub(super) fn up(&mut self) -> DotViewerResult<()> {
        match &self.focus {
            Focus::Current => {
                self.current.previous();
                self.update_adjacent()?
            }
            Focus::Prev => self.prevs.previous(),
            Focus::Next => self.nexts.previous(),
        }

        Ok(())
    }

    pub(super) fn down(&mut self) -> DotViewerResult<()> {
        match &self.focus {
            Focus::Current => {
                self.current.next();
                self.update_adjacent()?
            }
            Focus::Prev => self.prevs.next(),
            Focus::Next => self.nexts.next(),
        }

        Ok(())
    }

    pub(super) fn right(&mut self) {
        self.focus = match &self.focus {
            Focus::Current => Focus::Prev,
            Focus::Prev => Focus::Next,
            Focus::Next => Focus::Current,
        };
    }

    pub(super) fn left(&mut self) {
        self.focus = match &self.focus {
            Focus::Current => Focus::Next,
            Focus::Prev => Focus::Current,
            Focus::Next => Focus::Prev,
        };
    }

    pub(super) fn first(&mut self) -> DotViewerResult<()> {
        match &self.focus {
            Focus::Current => {
                self.current.first();
                self.update_adjacent()?
            }
            Focus::Prev => self.prevs.first(),
            Focus::Next => self.nexts.first(),
        }

        Ok(())
    }

    pub(super) fn last(&mut self) -> DotViewerResult<()> {
        match &self.focus {
            Focus::Current => {
                self.current.last();
                self.update_adjacent()?
            }
            Focus::Prev => self.prevs.last(),
            Focus::Next => self.nexts.last(),
        }

        Ok(())
    }
}
