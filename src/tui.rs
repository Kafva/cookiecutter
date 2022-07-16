use std::{io, thread, time::Duration, time::Instant};
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

// Desired functionality:
//  three level list menu:
//  [profile list] -> [domain list] -> [cookie list] -> [field list (view only)]
//  Global keymappings: 
//  h/j/k/l : Movement
//  D       : Delete current item, (Not valid at profile level)
pub fn run() -> Result<(),io::Error> {
    // Disable certain parts of the terminal's default behaviour
    //  https://docs.rs/crossterm/0.23.2/crossterm/terminal/index.html#raw-mode
    enable_raw_mode()?; 
    let mut stdout = std::io::stdout();

    // Enter fullscreen (crossterm API)
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;
    let tick_rate = Duration::from_millis(250);
    let mut app = App::new();

    run_ui(&mut terminal, &mut app, tick_rate).unwrap();

    thread::sleep(Duration::from_millis(5000));

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

struct App<'a> {
    items: StatefulList<(&'a str, usize)>
}
impl<'a> App<'a> {
    fn new() -> App<'a> {
        App {
            items: StatefulList::with_items(vec![
                ("Item0", 1),
                ("Item1", 2),
                ("Item2", 1),
                ("Item3", 3),
                ("Item4", 1),
                ("Item5", 4),
                ("Item6", 1),
                ("Item7", 3),
                ("Item8", 1),
                ("Item9", 6),
                ("Item10", 1),
                ("Item11", 3),
                ("Item12", 1),
                ("Item13", 2),
                ("Item14", 1),
                ("Item15", 1),
                ("Item16", 4),
                ("Item17", 1),
                ("Item18", 5),
                ("Item19", 4),
                ("Item20", 1),
                ("Item21", 2),
                ("Item22", 1),
                ("Item23", 3),
                ("Item24", 1),
            ]),
        }
    }
}


fn run_ui<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    tick_rate: Duration
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f,app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Left => app.items.unselect(),
                    KeyCode::Down => app.items.next(),
                    KeyCode::Up => app.items.previous(),
                    _ => {}
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

//cookie_dbs: &HashSet<CookieDB>
fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // Create two chunks with equal horizontal screen space
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
                     Constraint::Percentage(50), 
                     Constraint::Percentage(50)]
                     .as_ref())
        .split(f.size());

    let items: Vec<ListItem> = app.items.items.iter().map(|_| {
        ListItem::new("LOL").style(Style::default()
                                   .fg(Color::Black).bg(Color::White))
    }).collect();

    // Create a List from all list items and highlight the 
    // currently selected one
    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("List"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    // We can now render the item list
    f.render_stateful_widget(items, chunks[0], &mut app.items.state);
}

