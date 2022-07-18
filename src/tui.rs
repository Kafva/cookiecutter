use std::{
    io,
    io::Write,
    time::Duration, time::Instant,
    fs::OpenOptions,
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    text::Span,
    style::{Color, Modifier, Style},
    widgets::{
        Block, Borders, List, ListItem, Cell, Row, Table,
        BorderType, Paragraph
    },
    Frame, Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::{
    config::{
        DEBUG_LOG,
        NO_SELECTION,
        TUI_PRIMARY_COLOR,
        TUI_TEXT_TRUNCATE_LIM,
        TUI_SEARCH,
        Config
    },
    cookie_db::CookieDB,
    state::{State,Selection}
};

/// Entrypoint for the TUI
pub fn run(cookie_dbs: Vec<CookieDB>) -> Result<(),io::Error> {
    // Disable certain parts of the terminal's default behaviour
    //  https://docs.rs/crossterm/0.23.2/crossterm/terminal/index.html#raw-mode
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();

    // Enter fullscreen (crossterm API)
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;
    let tick_rate = Duration::from_millis(250);
    let mut state = State::new(&cookie_dbs);

    run_ui(&mut terminal, &mut state, cookie_dbs, tick_rate).unwrap();

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
 mut cookie_dbs: Vec<CookieDB>,
 tick_rate: Duration) -> io::Result<()> {
    let mut last_tick = Instant::now();

    // Auto-select the first profile
    if state.profiles.items.len() > 0 {
        state.profiles.status.select(Some(0));
    }

    loop {
        term.draw(|f| ui(f,state,&cookie_dbs))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if state.search_open {
                    //== Input mode ==//
                    handle_search_key(key.code, state, &cookie_dbs)
                } else {
                    //== Normal mode ==//
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        _ => handle_key(key.code, state, &mut cookie_dbs)
                    }
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

fn handle_search_key(code: KeyCode, state: &mut State,
 cookie_dbs: &Vec<CookieDB>) {
    match code {
        KeyCode::Enter => {
            state.search_open = false;
            state.search_matches.clear();
            let query: String = state.search_field.drain(..).collect();

            match state.selection {
                Selection::Profiles => {
                    // Save all partial matches
                    for (i,p) in cookie_dbs.iter().enumerate() {
                        if p.path.to_string_lossy().contains(&query) {
                            state.search_matches.push(i);
                        }
                    }
                    debug_log(format!("Search matches: {:?}",
                                      state.search_matches)
                    );
                    // Move selection to the first match (if any)
                    if state.search_matches.len() > 0 {
                        state.selected_match = 0;
                        state.profiles.status.select(
                            Some(*state.search_matches.get(0).unwrap())
                        );
                    }
                },
                Selection::Domains => {
                    if set_matches(
                        &state.current_domains.items,
                        query,
                        &mut state.search_matches
                    ) {
                        state.selected_match = 0;
                        state.current_domains.status.select(
                            Some(*state.search_matches.get(0).unwrap())
                        );
                    }
                },
                Selection::Cookies => {
                    if set_matches(
                        &state.current_cookies.items,
                        query,
                        &mut state.search_matches
                    ) {
                        state.selected_match = 0;
                        state.current_cookies.status.select(
                            Some(*state.search_matches.get(0).unwrap())
                        );
                    }
                }
            }
        }
        KeyCode::Char(c) => {
            state.search_field.push(c);
        }
        KeyCode::Backspace => {
            state.search_field.pop();
        }
        KeyCode::Esc => {
            state.search_field.drain(..);
            state.search_open = false
        }
        _ => {  }
    }
}

/// Handle keyboard input
fn handle_key(code: KeyCode, 
 state: &mut State, cookie_dbs: &mut Vec<CookieDB>) {
    match code {
        //== Deselect the current split ==//
        KeyCode::Left|KeyCode::Char('h') => {
            match state.selection {
                Selection::Profiles => {  }
                Selection::Domains => {
                    state.current_domains.status.select(None);
                    state.search_matches.clear();
                    state.selection = Selection::Profiles;
                }
                Selection::Cookies => {
                    state.current_cookies.status.select(None);
                    state.search_matches.clear();
                    state.selection = Selection::Domains;
                }
            }
        },
        //== Go to next item in split ==//
        KeyCode::Down|KeyCode::Char('j') => {
            match state.selection {
                Selection::Profiles => { state.profiles.next() }
                Selection::Domains => {
                  state.current_domains.next()
                }
                Selection::Cookies => {
                  // Cycle through cookies when the field
                  // window is selected
                  state.current_cookies.next()
                },
            }
        },
        //== Go to previous item in split ==//
        KeyCode::Up|KeyCode::Char('k') => {
            match state.selection {
                Selection::Profiles => { state.profiles.previous() }
                Selection::Domains => {
                    state.current_domains.previous()
                }
                Selection::Cookies => {
                    // Cycle through cookies when the field
                    // window is selected
                    state.current_cookies.previous()
                }
            }
        },
        //== Select the next split ==//
        KeyCode::Right|KeyCode::Char('l') => {
           match state.selection {
               Selection::Profiles => {
                    if state.current_domains.items.len() > 0 {
                        state.current_domains.status.select(Some(0));
                        state.search_matches.clear();
                        state.selected_match = NO_SELECTION;
                        state.selection = Selection::Domains;
                    }
               },
               Selection::Domains => {
                    if state.current_cookies.items.len() > 0 {
                        state.current_cookies.status.select(Some(0));
                        state.search_matches.clear();
                        state.selected_match = NO_SELECTION;
                        state.selection = Selection::Cookies;
                    }
               }
               Selection::Cookies => {
                    // The `state.current_fields.items` array is empty
                    // until the next ui() tick.
                    // This branch is needed to make the `match` exhaustive.
               }
           }
        },
        //== Select field through search ==//
        KeyCode::Char('/') => {
            state.search_open = true
        },
        //== Go to next match (if any) ==//
        KeyCode::Char('n') => {
            if state.search_matches.len() > 0 {
                // Wrap around if the last match has been reached
                state.selected_match =
                    if state.selected_match != state.search_matches.len()-1 {
                        state.selected_match+1
                    } else {
                        0
                    };
                select_match_in_current_split(state)
            }
        },
        //== Go to previous match (if any) ==//
        KeyCode::Char('N') => {
            if state.search_matches.len() > 0 {
                // Wrap around if the first match has been reached
                state.selected_match =
                    if state.selected_match != 0 {
                        state.selected_match-1
                    } else {
                        state.search_matches.len()-1
                    };
                select_match_in_current_split(state)
            }
        },
        //== Delete cookie(s) ==//
        KeyCode::Char('D') => {
            // Clear searches since any previously saved indices
            // will become incorrect
            state.search_matches.clear();
            state.selected_match = NO_SELECTION;
            delete_in_current_split(state,cookie_dbs)
        },
        //== Copy value to clipboard ==//
        KeyCode::Char('C') => {
            // pbcopy passthru
        },
        _ => {  }
    }
}

/// Delete the currently selected cookie if in the `Cookies` split
/// and all cookies from a domain if inside the `Domains` split
/// To update the internal cookie_db requires a mutable reference
fn delete_in_current_split(state: &mut State, cookie_dbs: &mut Vec<CookieDB>) {
    if let Some(profile_idx) = state.profiles.status.selected() {
        if let Some(cdb) = cookie_dbs.get_mut(profile_idx) {
            if let Some(current_domain) = state.selected_domain() {
                match state.selection {
                    // Remove all cookies from the current domain
                    Selection::Domains => {
                        debug_log(format!("Deleting: {current_domain}"));
                        cdb.delete_from_domain(
                            &current_domain, ""
                        ).expect("Failed to delete cookies from domain");

                        // If the removed item was the last domain,
                        // unselect the domains split
                        if state.current_domains.items.len() <= 1 {
                            state.current_domains.status.select(None)
                        }
                        // The selected() index needs to be decremented
                        // in case we removed that last item
                        else {
                            let curr = state.current_domains.status
                                .selected().unwrap();
                            if curr != 0 {
                                state.current_domains.status
                                    .select(Some(curr-1));
                            }
                        }
                    },
                    // Remove a specific cookie from the current domain
                    Selection::Cookies => {
                        if let Some(current_cookie) =
                         state.selected_cookie() {
                            debug_log(format!(
                                "Deleting: {current_domain}.{current_cookie}"
                            ));
                            cdb.delete_from_domain(
                                &current_domain, &current_cookie
                            ).expect("Failed to delete cookie");

                            // If the removed item was the last cookie,
                            // unselect the cookie split
                            if state.current_cookies.items.len() <= 1 {
                                state.current_cookies.status.select(None);
                                // If the domain split could also become empty
                                // from the deletion operation, select profiles
                                if state.current_domains.items.len() <= 1 {
                                    state.current_domains.status.select(None)
                                }
                            }
                            // The selected() index needs to be decremented
                            // in case we removed that last item
                            else {
                                let curr = state.current_cookies.status
                                    .selected().unwrap();
                                if curr != 0 {
                                    state.current_cookies.status
                                        .select(Some(curr-1));
                                }
                            }

                        }
                    },
                    _ => {  }
                }
            }
        }
    }
}

/// The `selected_match` is an index in the `search_matches` array, the
/// `search_matches` array contains the indices in the actual list.
fn select_match_in_current_split(state: &mut State){
    let list_idx = state.search_matches.get(state.selected_match)
        .expect("Invalid index specified for `search_matches`");

    debug_log(format!(
        "Selecting match[{}] -> list[{}]", state.selected_match, list_idx)
    );

    match state.selection {
        Selection::Profiles => {
            state.profiles.status.select(Some(*list_idx))
        },
        Selection::Domains => {
            state.current_domains.status.select(Some(*list_idx))
        },
        Selection::Cookies => {
            state.current_cookies.status.select(Some(*list_idx))
        }
    }
}

/// Save all partial matches of the query to `search_matches` and
/// return true if at least one match was found
fn set_matches(items: &Vec<String>, q: String, search_matches: &mut Vec<usize>)
 -> bool {
    for (i,p) in items.iter().enumerate() {
        if p.contains(&q) {
            search_matches.push(i);
        }
    }
    debug_log(format!("Search matches: {:?}", search_matches));
    search_matches.len() != 0
}

/// Render the UI, called on each tick.
/// Lists will be displayed at different indices depending on
/// which of the two views are active:
///  View 1: (selected 0-1)
///  View 2: (selected: 2)
///
///  |0       |1      |2           |3         |
///  |profiles|domains|cookie names|field_list|
///
fn ui<B: Backend>(frame: &mut Frame<B>, state: &mut State, cookie_dbs: &Vec<CookieDB>) {
    // == Layout ==//
    // Split the frame vertically into a body and footer
    let vert_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(98),
            Constraint::Percentage(2)]
        .as_ref())
        .split(frame.size());

    // Create three chunks for the body
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([
             Constraint::Percentage(33),
             Constraint::Percentage(33),
             Constraint::Percentage(33)
        ].as_ref())
        .split(vert_chunks[0]);

    if state.search_open {
        //== Render the search input ==//
        render_search(frame, state, vert_chunks[1])
    } else {
        //== Render the footer ==//
        frame.render_widget(create_footer(), vert_chunks[1])
    }

    // Determine which splits should be rendered
    let (profiles_idx, domains_idx, cookies_idx, fields_idx) =
        if matches!(state.selection, Selection::Cookies) {
            (NO_SELECTION,0,1,2)
        } else {
            (0,1,2,NO_SELECTION)
        };

    if profiles_idx != NO_SELECTION {
        //== Profiles ==//
        let profile_items: Vec<ListItem> =
            create_list_items(&state.profiles.items);

        let profile_list =  add_highlight(
            create_list(profile_items,
                "Profiles".to_string(), Borders::NONE
            )
        );

        //== Render profiles ==//
        frame.render_stateful_widget(
            profile_list, chunks[profiles_idx], &mut state.profiles.status
        );
    }

    //== Domains ==//
    if let Some(profile_idx) = state.profiles.status.selected() {
        if let Some(cdb) = cookie_dbs.get(profile_idx) {
            // Fill the current_domains state list
            state.current_domains.items = cdb.domains();

            let domain_items = create_list_items(&state.current_domains.items);

            let domain_list = add_highlight(
                create_list(domain_items, "Domains".to_string(), Borders::NONE)
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
                        .map(|c| c.name.to_owned() ).collect();

                let cookies_items = create_list_items(
                    &state.current_cookies.items
                );

                let cookies_list = add_highlight(
                    create_list(cookies_items,
                        "Cookies".to_string(),
                        Borders::NONE
                ));

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
                        let fields_items: Vec<ListItem> =
                            create_list_items(&state.current_fields.items);

                        let fields_list = create_list(
                            fields_items, "Fields".to_string(), Borders::ALL
                        );

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

/// Create list items for the UI
/// Nodes with text exceeding `TUI_TEXT_TRUNCATE_LIM`
/// will be truncated with `...`
fn create_list_items<T: ToString>(items: &Vec<T>) -> Vec<ListItem> {
    items.iter().map(|p| {
        let p: String = p.to_string();
        let text = if p.len() > TUI_TEXT_TRUNCATE_LIM {
            format!("{}..", &p[0..TUI_TEXT_TRUNCATE_LIM])
        } else {
            p
        };
        ListItem::new(text)
    }).collect()
}

fn render_search<B: Backend>(
 frame: &mut Frame<B>, state: &mut State, vert_chunk: Rect) {
    let input_box = Paragraph::new(
       format!("{} {}", TUI_SEARCH, state.search_field)
    ).style(Style::default().fg(Color::Blue));

    frame.render_widget(input_box, vert_chunk);
    frame.set_cursor(
        // Put cursor past the end of the input text
        vert_chunk.x +
            TUI_SEARCH.len() as u16 +
            state.search_field.len() as u16 +
            1,
        vert_chunk.y,
    );
}

/// Create the usage footer
fn create_footer() -> Table<'static> {
    let cells = [
        Cell::from("/: Search")
            .style(Style::default().fg(Color::LightBlue)),
        Cell::from("n/N: Next/Previous match"),
        Cell::from("D: Delete")
            .style(Style::default().fg(Color::LightRed)),
        Cell::from("C: Copy to clipboard")
            .style(Style::default().fg(Color::LightYellow)),
        Cell::from("q: Quit")
    ];

    let row = Row::new(cells).bottom_margin(1);
    Table::new(vec![row])
        .block(Block::default().borders(Borders::NONE))
        .widths(&[
            Constraint::Percentage(7),
            Constraint::Percentage(15),
            Constraint::Percentage(7),
            Constraint::Percentage(12),
            Constraint::Percentage(7),
        ])
}

/// Highlighted the currently selected item
fn add_highlight(list: List) -> List {
    list.highlight_style(
        Style::default()
            .fg(Color::Indexed(TUI_PRIMARY_COLOR))
            .add_modifier(Modifier::BOLD),
    )
}

/// Create a TUI `List` from a `ListItem` vector
fn create_list(items: Vec<ListItem>, title: String, border: Borders) -> List {
    List::new(items)
        .block(
            Block::default().border_type(BorderType::Rounded).borders(border)
            .title(Span::styled(title,
                    Style::default().fg(Color::Indexed(TUI_PRIMARY_COLOR))
                        .add_modifier(Modifier::UNDERLINED|Modifier::BOLD)
                )
            )
        )
}

/// Print a debug message to `DEBUG_LOG`
#[allow(dead_code)]
fn debug_log<T: std::fmt::Display>(msg: T) {
    if Config::global().debug {
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(DEBUG_LOG)
            .unwrap();

        writeln!(f,"-> {msg}").expect("Failed to write debug message");
    }
}

