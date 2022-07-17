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
    config::{DEBUG_LOG,INVALID_SPLIT_ERR,TUI_SELECTED_ROW,NO_SELECTION,ALL_FIELDS},
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
fn run_ui<B: Backend>(term: &mut Terminal<B>, state: &mut State,
 tick_rate: Duration) -> io::Result<()> {
    let mut last_tick = Instant::now();

    // Auto-select the first profile: TODO handle no-profiles
    state.profiles.state.select(Some(0));

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
                  let dms = state.domains_for_profile();
                  if let Some(dms) = dms {
                      dms.state.select(None);
                      state.selected_split -= 1
                  }
                }
                2 => {
                  let cks = state.cookies_for_domain();
                  if let Some(cks) = cks {
                      cks.state.select(None);
                      state.selected_split -= 1
                  }
                }
               _ => panic!("{}", INVALID_SPLIT_ERR)
            }

        },
        //== Go to next item in split ==//
        KeyCode::Down|KeyCode::Char('j') => {
            match state.selected_split {
                0 => { state.profiles.next() }
                1 => {
                  let dms = state.domains_for_profile();
                  if let Some(dms) = dms {
                      dms.next();
                  }
                }
                2 => {
                  let cks = state.cookies_for_domain();
                  if let Some(cks) = cks {
                      cks.next()
                  }
                },
                3 => {

                }
               _ => panic!("{}", INVALID_SPLIT_ERR)
            }
        },
        //== Go to previous item in split ==//
        KeyCode::Up|KeyCode::Char('k') => {
            match state.selected_split {
                0 => { state.profiles.previous() }
                1 => {
                  let dms = state.domains_for_profile();
                  if let Some(dms) = dms {
                      dms.previous();
                  }
                }
                2 => {
                  let cks = state.cookies_for_domain();
                  if let Some(cks) = cks {
                      cks.previous()
                  }
                }
               _ => panic!("{}", INVALID_SPLIT_ERR)
            }
        },
        //== Select the next split ==//
        KeyCode::Right|KeyCode::Char('l') => {
            if state.selected_split < 2 {
               // if_let chaining is used to ensure
               // that the next split has at least one item
               // before switching
               match state.selected_split {
                   0 => {
                      let dms = state.domains_for_profile();
                      if let Some(dms) = dms && dms.items.len() > 0 {
                          dms.state.select(Some(0));
                          state.selected_split+=1;
                      }
                   },
                   1 => {
                      let cks = state.cookies_for_domain();
                      if let Some(cks) = cks && cks.items.len() > 0 {
                          cks.state.select(Some(0));
                          state.selected_split+=1;
                      }
                   }
                   _ => panic!("{}", INVALID_SPLIT_ERR)
               }

            }
        },
        _ => {  }
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


    // Lists will be displayed at different indices depending on if
    // which of the two views are active:
    //  View 1:
    //  |profiles|domains|cookie names|
    //
    //  View 2:
    //  |domains|cookie names|field list|
    let (profiles_idx, domains_idx, cookies_idx, fields_idx) = 
        if state.selected_split < 3 {
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
            profile_list, chunks[profiles_idx], &mut state.profiles.state
        );
    }

    //== Domains ==//
    if let Some(domains) = state.domains_for_profile() {
        let domain_items: Vec<ListItem> = domains
            .items.iter().map(|p| {
                ListItem::new(*p).style(Style::default())
        }).collect();

        let domain_list = create_list(domain_items, String::from("Domains"));

        //== Render domains ==//
        frame.render_stateful_widget(
            domain_list, chunks[domains_idx], &mut domains.state
        );

        //== Cookies ==//
        if let Some(cookies) = state.cookies_for_domain() {
            let cookies_items: Vec<ListItem> = cookies.items.iter()
                .map(|c| {
                    ListItem::new(c.name.as_str()).style(Style::default())
            }).collect();

            let cookies_list =
                create_list(cookies_items, String::from("Cookies"));

            //== Render cookies ==//
            frame.render_stateful_widget(
                cookies_list, chunks[cookies_idx], &mut cookies.state
            );

            //== Fields ==//
            //if let Some(current_cookie) = state.fields_for_cookie() {

            //    let fields_items = current_cookie.as_list_items();

            //    let fields_list =
            //        create_list(fields_items, String::from("Fields"));

            //    let tmp = current_cookie
            //            .fields_as_str(
            //                &String::from(ALL_FIELDS),
            //                true,
            //                false
            //            ).split("\n");

            //    state.current_fields.items = tmp.collect();

            //    //== Render fields ==//
            //    frame.render_stateful_widget(
            //        fields_list, chunks[fields_idx], &mut state.current_fields.state
            //    );

            //}
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

