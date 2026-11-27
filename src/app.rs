use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::update::{self, CheckResult, UpdateEvent, UpdateState};

const CHECK_INTERVAL: Duration = Duration::from_secs(30);

/// Keep in sync with the chapter list in `src/ui/chapters.rs`.
const CHAPTER_COUNT: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Home,
    Chapters,
    /// A chapter detail page. The value is the index into the chapter list.
    Chapter(usize),
}
pub struct App {
    pub screen: Screen,
    /// Index of the highlighted chapter on the Chapters screen.
    pub selected_chapter: usize,
    /// Vertical scroll offset of the open chapter detail page.
    pub chapter_scroll: u16,
    pub state: UpdateState,
    pub should_quit: bool,
    pub restart: bool,
    etag: Option<String>,
    receiver: Option<Receiver<UpdateEvent>>,
    last_check: Instant,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            screen: Screen::Home,
            selected_chapter: 0,
            chapter_scroll: 0,
            state: UpdateState::Idle,
            should_quit: false,
            restart: false,
            etag: None,
            receiver: None,
            last_check: Instant::now(),
        };
        app.check();
        app
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('c') => self.should_quit = true,
                KeyCode::Char('u') => self.update_action(),
                _ => {}
            }
            return;
        }

        match self.screen {
            Screen::Home => match key.code {
                KeyCode::Enter => self.screen = Screen::Chapters,
                KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                _ => {}
            },
            Screen::Chapters => match key.code {
                KeyCode::Up => {
                    self.selected_chapter = self.selected_chapter.saturating_sub(1);
                }
                KeyCode::Down => {
                    self.selected_chapter =
                        (self.selected_chapter + 1).min(CHAPTER_COUNT.saturating_sub(1));
                }
                KeyCode::Enter => {
                    self.chapter_scroll = 0;
                    self.screen = Screen::Chapter(self.selected_chapter);
                }
                KeyCode::Esc | KeyCode::Backspace => self.screen = Screen::Home,
                KeyCode::Char('q') => self.should_quit = true,
                _ => {}
            },
            Screen::Chapter(_) => match key.code {
                KeyCode::Up => {
                    self.chapter_scroll = self.chapter_scroll.saturating_sub(1);
                }
                KeyCode::Down => {
                    self.chapter_scroll = self.chapter_scroll.saturating_add(1);
                }
                KeyCode::Esc | KeyCode::Backspace => self.screen = Screen::Chapters,
                KeyCode::Char('q') => self.should_quit = true,
                _ => {}
            },
        }
    }

    pub fn tick(&mut self) {
        self.poll();

        let idle = matches!(self.state, UpdateState::Idle | UpdateState::Failed { .. });
        if idle && self.receiver.is_none() && self.last_check.elapsed() >= CHECK_INTERVAL {
            self.check();
        }
    }

    fn update_action(&mut self) {
        match &self.state {
            UpdateState::Available { release } => {
                let release = release.clone();
                self.state = UpdateState::Downloading {
                    latest: release.version.clone(),
                    downloaded: 0,
                    total: None,
                };
                self.receiver = Some(update::install(release));
            }
            UpdateState::Ready { .. } => self.restart = true,
            UpdateState::Failed { .. } => self.check(),
            _ => {}
        }
    }

    fn check(&mut self) {
        self.last_check = Instant::now();
        self.receiver = Some(update::check(self.etag.clone()));
    }

    fn poll(&mut self) {
        let mut events = Vec::new();
        let mut disconnected = false;

        if let Some(receiver) = &self.receiver {
            loop {
                match receiver.try_recv() {
                    Ok(event) => events.push(event),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        disconnected = true;
                        break;
                    }
                }
            }
        }

        if disconnected {
            self.receiver = None;
        }

        for event in events {
            self.handle(event);
        }
    }

    fn handle(&mut self, event: UpdateEvent) {
        match event {
            UpdateEvent::Checked { result, etag } => {
                self.receiver = None;
                self.etag = etag;
                match result {
                    CheckResult::Available(release) => {
                        self.state = UpdateState::Available { release };
                    }
                    CheckResult::UpToDate => self.state = UpdateState::Idle,
                    CheckResult::Error => self.state = UpdateState::Idle,
                }
            }
            UpdateEvent::Progress { downloaded, total } => {
                if let UpdateState::Downloading { latest, .. } = &self.state {
                    self.state = UpdateState::Downloading {
                        latest: latest.clone(),
                        downloaded,
                        total,
                    };
                }
            }
            UpdateEvent::Installed => {
                if let UpdateState::Downloading { latest, .. } = &self.state {
                    self.state = UpdateState::Ready {
                        latest: latest.clone(),
                    };
                }
                self.receiver = None;
            }
            UpdateEvent::Failed(message) => {
                self.state = UpdateState::Failed { message };
                self.receiver = None;
            }
        }
    }
}
