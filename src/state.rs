use tui::widgets::ListState;

use crate::cookie_db::CookieDB;
use crate::config::NO_SELECTION;

pub struct StatefulList<T> {
    pub status: ListState,
    pub items: Vec<T>,
}

/// https://github.com/fdehau/tui-rs/tree/master/examples/list.rs
impl<T> StatefulList<T> {
    pub fn default() -> Self {
        StatefulList { status: ListState::default(), items: vec![] }
    }
    pub fn next(&mut self) {
        let i = match self.status.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.status.select(Some(i));
    }
    pub fn previous(&mut self) {
        let i = match self.status.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.status.select(Some(i));
    }
}

#[derive(PartialEq)]
pub enum Selection {
    Profiles,
    Domains,
    Cookies
}

/// The main struct which holds the global state of the TUI
pub struct State {
    /// The currently selected element
    pub selection: Selection, 

    pub search_open: bool,
    pub search_field: String,

    /// Indices of all matches from a '/' search
    pub search_matches: Vec<usize>,

    /// The index in the search results
    pub selected_match: usize,

    // We we only keep the domains for the currently seleceted profile
    // in a StatefulList. If a domain is removed, we will update the
    // underlying CookieDB and reload
    // pub cookie_dbs: Vec<CookieDB>,

    pub profiles:        StatefulList<String>,
    pub current_domains: StatefulList<String>,
    pub current_cookies: StatefulList<String>,
    pub current_fields:  StatefulList<String>
}

impl State {
    /// Create a TUI state object from a vector of cookie databases
    pub fn new(cookie_dbs: &Vec<CookieDB>) -> State {
        // The profiles list will never change after launch
        let profiles = StatefulList {
            status: ListState::default(),
            items: cookie_dbs.iter().map(|c| {
                c.path_short()
            }).collect()
        };
        State {
            selection: Selection::Profiles, 
            search_open: false,
            search_field: "".to_string(),
            search_matches: vec![],
            selected_match: NO_SELECTION,
            profiles, 
            current_domains: StatefulList::default(), 
            current_cookies: StatefulList::default(), 
            current_fields:  StatefulList::default(),
        }
    }

    /// The currently selected profile 
    pub fn selected_profile(&self) -> Option<String> {
        if let Some(selected_idx) = self.profiles.status.selected() {
            // Convert to String to dodge BC
            let s = self.profiles.items.get(selected_idx)
                .expect("No profile found for `selected()` index");
            Some(s.to_owned())
        } else {
            None
        }
    }

    /// The currently selected domain (if any)
    pub fn selected_domain(&self) -> Option<String> {
        if let Some(selected_idx) = self.current_domains.status.selected() {
            // Convert to String to dodge BC
            let s = self.current_domains.items.get(selected_idx)
                .expect("No domain found for `selected()` index");
            Some(s.to_owned())
        } else {
            None
        }
    }

    /// The currently selected cookie (if any)
    pub fn selected_cookie(&self) -> Option<String> {
        if let Some(selected_idx) = self.current_cookies.status.selected() {
            // Convert to String to dodge BC
            let c = self.current_cookies.items.get(selected_idx)
                .expect("No cookie found for `selected()` index");
            Some(c.to_owned())
        } else {
            None
        }
    }
}

