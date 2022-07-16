use std::{
    io, time::Duration, time::Instant,
    collections::HashMap
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame, Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};


use crate::types::{CookieDB,Cookie};

pub fn run(cookie_dbs: &Vec<CookieDB>) -> Result<(),io::Error> {
    // Disable certain parts of the terminal's default behaviour
    //  https://docs.rs/crossterm/0.23.2/crossterm/terminal/index.html#raw-mode
    enable_raw_mode()?; 
    let mut stdout = std::io::stdout();

    // Enter fullscreen (crossterm API)
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;
    let tick_rate = Duration::from_millis(250);
    let mut state = State::from_cookie_dbs(&cookie_dbs);

    run_ui(&mut terminal, &mut state, tick_rate).unwrap();

    // Restore default terminal behaviour
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}


//============================================================================//
struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

// Desired functionality:
//  leveled list menu:
//  [profile list] -> [domain list] -> [cookie list] -> [field list (view only)]
//  Global keymappings: 
//  h/j/k/l : Movement
//  D       : Delete current item, (Not valid at profile level)
//
//  View 1:
//  |profiles|domains|cookie names|
//
//  View 2:
//  |domains|cookie names|field list|

/// The main struct which holds the global state of the TUI
struct State<'a> {
    selected_split: u32,
    profiles: StatefulList<String>,

    // The cookies and domain lists will need to be updatable
    // profile        -> stateful list of domains
    // profile+domain -> stateful list of cookies
    domains: HashMap<String,StatefulList<&'a str>>,
    cookies: HashMap<String,StatefulList<&'a Cookie>>,
}
impl<'a> State<'a> {
    fn from_cookie_dbs(cookie_dbs: &Vec<CookieDB>) -> State {
        // Statefil list of profiles
        let profiles = StatefulList::with_items( 
            cookie_dbs.iter().map(|c| c.path_short()).collect()
        );
        let mut domains = HashMap::new();
        let mut cookies = HashMap::new();

        for cdb in cookie_dbs {
           let mut hst_names: Vec<&str> = 
               cdb.cookies.iter().map(|c| c.host.as_str()).collect();
           hst_names.sort();
           hst_names.dedup();

           for hst_name in hst_names.as_slice() {
                let cookies_for_hst = cdb.cookies.iter().filter(|c|
                   c.host == **hst_name 
                ).collect();

                // Statefil list of cookies (per domain, per profile)
                cookies.insert(
                    cdb.path_short()+&hst_name,
                    StatefulList::with_items(cookies_for_hst)
                );
           }

            // Statefil list of domains (per profile)
           domains.insert(
              cdb.path_short(), 
              StatefulList::with_items(hst_names)
           );

        }

        State {
            selected_split: 0, profiles, domains, cookies
        }
    }
}


fn run_ui<B: Backend>(
    terminal: &mut Terminal<B>,
    state: &mut State,
    tick_rate: Duration
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f,state))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Left => state.profiles.unselect(),
                    KeyCode::Down => state.profiles.next(),
                    KeyCode::Up => state.profiles.previous(),
                    _ => {}
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(frame: &mut Frame<B>, state: &mut State) {
    // Create two chunks with equal horizontal screen space
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
             Constraint::Percentage(33), 
             Constraint::Percentage(33),
             Constraint::Percentage(33)
        ].as_ref()).split(frame.size());

    let items: Vec<ListItem> = state.profiles.items.iter().map(|p| {
        ListItem::new(p.as_str()).style(Style::default())
    }).collect();

    // Create a List from all list items and highlight the 
    // currently selected one
    let items = List::new(items)
        .block(Block::default().borders(Borders::RIGHT).title("Profiles"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    // We can now render the item list
    frame.render_stateful_widget(items, chunks[0], &mut state.profiles.state);
}

