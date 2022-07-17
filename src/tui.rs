use std::{
    io,
    io::Write,
    time::Duration, time::Instant,
    fs::OpenOptions,
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame, Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::{
    config::{DEBUG_LOG,INVALID_SPLIT_ERR,TUI_SELECTED_ROW,NO_SELECTION},
    cookie_db::CookieDB,
    state::State
};

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
    let mut state = State::from_cookie_dbs(cookie_dbs);

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
fn run_ui<B: Backend>(term: &mut Terminal<B>, state: &mut State,
 tick_rate: Duration) -> io::Result<()> {
    let mut last_tick = Instant::now();

    // Auto-select the first profile
    if state.profiles.items.len() > 0 {
        state.profiles.status.select(Some(0));
    }

    loop {
        term.draw(|f| ui(f,state))?;

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

/// Handle keyboard input
fn handle_key(code: KeyCode, state: &mut State) {
    match code {
        //== Deselect the current split ==//
        KeyCode::Left|KeyCode::Char('h') => {
            match state.selected_split {
                0 => {  }
                1 => {
                    state.current_domains.status.select(None);
                    state.selected_split-=1;
                }
                2 => {
                    state.current_cookies.status.select(None);
                    state.selected_split-=1;
                }
               _ => panic!("{}", INVALID_SPLIT_ERR)
            }

        },
        //== Go to next item in split ==//
        KeyCode::Down|KeyCode::Char('j') => {
            match state.selected_split {
                0 => { state.profiles.next() }
                1 => {
                  state.current_domains.next()
                }
                2|3 => {
                  // Cycle through cookies when the field
                  // window is selected
                  state.current_cookies.next()
                },
               _ => panic!("{}", INVALID_SPLIT_ERR)
            }
        },
        //== Go to previous item in split ==//
        KeyCode::Up|KeyCode::Char('k') => {
            match state.selected_split {
                0 => { state.profiles.previous() }
                1 => {
                    state.current_domains.previous()
                }
                2|3 => {
                    // Cycle through cookies when the field
                    // window is selected
                    state.current_cookies.previous()
                }
               _ => panic!("{}", INVALID_SPLIT_ERR)
            }
        },
        //== Select the next split ==//
        KeyCode::Right|KeyCode::Char('l') => {
           match state.selected_split {
               0 => {
                    if state.current_domains.items.len() > 0 {
                        state.current_domains.status.select(Some(0));
                        state.selected_split+=1;
                    }
               },
               1 => {
                    if state.current_cookies.items.len() > 0 {
                        state.current_cookies.status.select(Some(0));
                        state.selected_split+=1;
                    }
               }
               2 => {
                    // The `state.current_fields.items` array is empty
                    // until the next ui() tick.
                    // state.selected_split+=1;
               }
               _ => panic!("{}", INVALID_SPLIT_ERR)
           }
        },
        _ => {  }
    }
}

/// Render the UI, called on each tick
///
/// Lists will be displayed at different indices depending on if
/// which of the two views are active:
///  View 1: (selected 0-1)
///  View 2: (selected: 2)
///
///  |0       |1      |2           |3         |
///  |profiles|domains|cookie names|field_list|
///
fn ui<B: Backend>(frame: &mut Frame<B>, state: &mut State) {
    // Create two chunks with equal horizontal screen space
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
             Constraint::Percentage(33),
             Constraint::Percentage(33),
             Constraint::Percentage(33)
        ].as_ref()).split(frame.size());

    let (profiles_idx, domains_idx, cookies_idx, fields_idx) = 
        if state.selected_split < 2 {
            (0,1,2,NO_SELECTION)
        } else {
            (NO_SELECTION,0,1,2)
        };

    if profiles_idx != NO_SELECTION {
        //== Profiles ==//
        let profile_items: Vec<ListItem> = state.profiles.items.iter().map(|p| {
            ListItem::new(p.as_str()).style(Style::default())
        }).collect();

        let profile_list = create_list(profile_items, String::from("Profiles"));

        //== Render profiles ==//
        frame.render_stateful_widget(
            profile_list, chunks[profiles_idx], &mut state.profiles.status
        );
    }

    //== Domains ==//
    if let Some(profile_idx) = state.profiles.status.selected() {
        if let Some(cdb) = state.cookie_dbs.get(profile_idx) {
            // Fill the current_domains state list
            state.current_domains.items = cdb.domains();

            // Create list items for the UI
            let domain_items: Vec<ListItem> = cdb.domains().iter().map(|p| {
                    ListItem::new(p.to_owned()).style(Style::default())
            }).collect();
            let domain_list = create_list(
                domain_items, String::from("Domains")
            );

            //== Render domains ==//
            frame.render_stateful_widget(
                domain_list, chunks[domains_idx], 
                &mut state.current_domains.status
            );

            //== Cookies ==//
            if let Some(current_domain) = state.selected_domain() {
                // Fill the current_cookies state list
                state.current_cookies.items = 
                    cdb.cookies_for_domain(&current_domain).iter()
                        .map(|c| c.name.as_str() ).collect();

                // Create list items for the UI
                let cookies_items = state.current_cookies.items.iter().map(|c| {
                        ListItem::new(*c).style(Style::default())
                }).collect();
                let cookies_list = 
                    create_list(cookies_items, String::from("Cookies"));

                //== Render cookies ==//
                frame.render_stateful_widget(
                    cookies_list, chunks[cookies_idx], 
                    &mut state.current_cookies.status
                );

                //== Fields ==//
                if let Some(current_cookie) = state.selected_cookie() {
                    if let Some(cookie) = cdb
                        .cookie_for_domain(
                            &current_cookie,&current_domain
                        ) {

                        // Fill the current_fields state list
                        state.current_fields.items = vec![
                            cookie.match_field("Value",true,false),
                            cookie.match_field("Path",true,false),
                            cookie.match_field("Creation",true,false),
                            cookie.match_field("Expiry",true,false),
                            cookie.match_field("LastAccess",true,false),
                            cookie.match_field("HttpOnly",true,false),
                            cookie.match_field("Secure",true,false),
                            cookie.match_field("SameSite",true,false),
                        ];

                        // Create list items for the UI
                        let fields_items: Vec<ListItem> = state
                            .current_fields.items.iter().map(|f| {
                                ListItem::new(f.as_str())
                                    .style(Style::default())
                        }).collect();

                        let fields_list = List::new(fields_items)
                                .block(Block::default()
                                .title("Fields"));

                        if fields_idx != NO_SELECTION {
                            //== Render fields ==//
                            frame.render_stateful_widget(
                                fields_list, chunks[fields_idx], 
                                &mut state.current_fields.status
                            );
                            if state.current_fields.items.len() > 0 {
                                state.current_fields.status.select(Some(0));
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Create a TUI `List` from a `ListItem` vector
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
        .highlight_symbol(TUI_SELECTED_ROW)
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

