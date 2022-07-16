use std::{
    io,
    io::Write,
    time::Duration, time::Instant,
    collections::HashMap,
    fs::OpenOptions,
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

use crate::{
    config::{DEBUG_LOG,NO_SELECTION,INVALID_SPLIT_ERR},
    types::{CookieDB,Cookie}
};

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
//  Global key mappings:
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
    /// Create a TUI state object from a vector of cookie databases
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

    /// To avoid BC issues when borrowing other parts of the state,
    /// this method returns a cloned string instead of a borrowed reference
    fn _selected_profile(&self) -> Option<String> {
        let selected: usize = self.profiles.state.selected()
                .unwrap_or_else(|| NO_SELECTION);
        if selected != NO_SELECTION {
            Some(self.profiles.items.get(selected).unwrap().clone())
        } else {
           None
        }

    }

    /// Fetch the `StatefulList` of domains for the currently selected profile
    fn domains_for_profile(&mut self) -> Option<&mut StatefulList<&'a str>> {
        let selected: usize = self.profiles.state.selected()
                .unwrap_or_else(|| NO_SELECTION);
        if selected != NO_SELECTION {
            // Note that the reference is mutable, this is required to call
            // e.g. `select()` and `unselect()`
            self.domains.get_mut(self.profiles.items.get(selected).unwrap())
        } else {
           None
        }
    }

    /// Fetch the `StatefulList` of cookies for the currently selected domain 
    /// of a specific profile
    fn cookies_for_domain(&mut self) -> Option<&mut StatefulList<&'a Cookie>> {
        let current_profile = self._selected_profile();
        if current_profile.is_some() {
            // .clone() to dodge BC
            let current_profile = current_profile.unwrap().clone();

            let current_domains = self.domains_for_profile();
            if current_domains.is_some() {
                let current_domains = current_domains.unwrap();
                let selected_idx = current_domains.state.selected().unwrap();
                let current_domain = // .clone() to dodge BC
                    current_domains.items.get(selected_idx).unwrap().clone();

                let key = format!("{}{}", current_profile, current_domain);
                self.cookies.get_mut(&key)
            } else {
                None
            }
        } else {
            None
        }
    }

}

//============================================================================//

/// Handle keyboard input
fn handle_key(code: KeyCode, state: &mut State) {
    match code {
        KeyCode::Left|KeyCode::Char('h') => {
            // TODO call unselect() on the correct list
            match state.selected_split {
                0 => {  }
                1 => { 
                  let domain = state.domains_for_profile();
                  if domain.is_some() {
                    domain.unwrap().unselect();
                    state.selected_split -= 1
                  }
                }
                2 => {  }
               _ => panic!("{}", INVALID_SPLIT_ERR)
            }

        },
        KeyCode::Down|KeyCode::Char('j') => {
            match state.selected_split {
                0 => { state.profiles.next() }
                1 => {  
                  let domain = state.domains_for_profile();
                  if domain.is_some() {
                      domain.unwrap().next()
                  }

                }
                2 => {  }
               _ => panic!("{}", INVALID_SPLIT_ERR)
            }
            // TODO call next on the correct list
        },
        KeyCode::Up|KeyCode::Char('k') => {
            match state.selected_split {
                0 => { state.profiles.previous() }
                1 => {  
                  let domain = state.domains_for_profile();
                  if domain.is_some() {
                      domain.unwrap().previous()
                  }
                }
                2 => {  }
               _ => panic!("{}", INVALID_SPLIT_ERR)
            }
            // TODO call previous on the correct list
        },
        KeyCode::Right|KeyCode::Char('l') => {
            if state.selected_split < 2 {
               match state.selected_split {
                   0 => {
                      let domain = state.domains_for_profile();
                      if domain.is_some() {
                          // TODO: handle case where no domains exist
                          domain.unwrap().state.select(Some(0));
                          state.selected_split+=1;
                      }
                   },
                   1 => {
                      state.selected_split+=1;
                      // Select the first item from the current
                      // `cookies_for_domain`
                   }
                   _ => panic!("{}", INVALID_SPLIT_ERR)
               }

            }
        },
        _ => {  }
    }
}

/// Entrypoint for the TUI
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

/// Application loop
fn run_ui<B: Backend>(
    terminal: &mut Terminal<B>,
    state: &mut State,
    tick_rate: Duration
) -> io::Result<()> {
    let mut last_tick = Instant::now();

    // Auto-select the first profile: TODO handle no-profiles
    state.profiles.state.select(Some(0));

    loop {
        terminal.draw(|f| ui(f,state))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    _ => handle_key(key.code, state)
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

/// Render the UI, called on each tick
fn ui<B: Backend>(frame: &mut Frame<B>, state: &mut State) {
    // Create two chunks with equal horizontal screen space
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
             Constraint::Percentage(33),
             Constraint::Percentage(33),
             Constraint::Percentage(33)
        ].as_ref()).split(frame.size());


    //== Profiles ==//
    let profile_items: Vec<ListItem> = state.profiles.items.iter().map(|p| {
        ListItem::new(p.as_str()).style(Style::default())
    }).collect();

    let profile_list = create_list(profile_items, String::from("Profiles"));

    //== Render profiles ==//
    frame.render_stateful_widget(
        profile_list, chunks[0], &mut state.profiles.state
    );

    //== Domains ==//
    // Add the domains of the selected profile to the second chunk
    //let selected_profile = state.profiles.items.get(selected).unwrap();

    // Note that `domains_for_profile` and `cookies_for_domain`
    // need to mutably borrowed to support updatates in the frame
    let domains = state.domains_for_profile();

    if domains.is_some() {
        let domains = domains.unwrap();

        let domain_items: Vec<ListItem> = domains
            .items.iter().map(|p| {
                ListItem::new(*p).style(Style::default())
        }).collect();

        let domain_list = create_list(domain_items, String::from("Domains"));

        //== Cookies ==//
        // Determine the currently selected domain
        //let selected: usize = domains.state.selected()
        //    .unwrap_or_else(|| NO_SELECTION);
        //if selected != NO_SELECTION {
        //    let selected_domain = domains.items
        //        .get(selected).unwrap();

        //    // Add the cookies of the selected domain to the third chunk
        //    let cookies_for_domain  = state.cookies.get_mut(
        //        &format!("{}{}", selected_profile, selected_domain)
        //    ).unwrap();

        let cookies = state.cookies_for_domain();
        if cookies.is_some() {
            let cookies = cookies.unwrap();
            let cookie_items: Vec<ListItem> = cookies.items.iter()
                .map(|c| {
                    ListItem::new(c.name.as_str()).style(Style::default())
            }).collect();

            let cookie_list =
                create_list(cookie_items, String::from("Cookies"));

            //== Render cookies ==//
            frame.render_stateful_widget(
                cookie_list, chunks[2], &mut cookies.state
            );
        }

        //== Render domains ==//
        frame.render_stateful_widget(
            domain_list, chunks[1], &mut domains.state
        );

    }

}

fn create_list(items: Vec<ListItem>, title: String) -> List {
    List::new(items)
        .block(Block::default().borders(Borders::RIGHT)
        .title(title))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ")
}

/// Print a debug message to `DEBUG_LOG`
#[allow(dead_code)]
fn debug_log<T: std::fmt::Display>(msg: T) {
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(DEBUG_LOG)
        .unwrap();

    writeln!(f,"-> {msg}").expect("Failed to write debug message");
}

