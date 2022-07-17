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
    config::{DEBUG_LOG,INVALID_SPLIT_ERR},
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
                    domain.unwrap().state.select(None);
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
    // `domains_for_profile` and `cookies_for_domain`
    // need to mutably borrowed to support updates in the frame
    let domains = state.domains_for_profile();

    if domains.is_some() {
        let domains = domains.unwrap();

        let domain_items: Vec<ListItem> = domains
            .items.iter().map(|p| {
                ListItem::new(*p).style(Style::default())
        }).collect();

        let domain_list = create_list(domain_items, String::from("Domains"));

        //== Render domains ==//
        frame.render_stateful_widget(
            domain_list, chunks[1], &mut domains.state
        );

        //== Cookies ==//
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

