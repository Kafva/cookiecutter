use tui::widgets::ListState;

use crate::cookie_db::CookieDB;

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


/// The main struct which holds the global state of the TUI
pub struct State<'a> {
    /// Valid range: 0 - 2
    pub selected_split: u8, 

    // We we only keep the domains for the currently seleceted profile
    // in a StatefulList. If a domain is removed, we will update the
    // underlying CookieDB and reload
    pub cookie_dbs: &'a Vec<CookieDB>,

    pub profiles:        StatefulList<String>,
    pub current_domains: StatefulList<&'a str>,
    pub current_cookies: StatefulList<&'a str>,
    pub current_fields:  StatefulList<String>
}

impl<'a> State<'a> {
    /// Create a TUI state object from a vector of cookie databases
    pub fn from_cookie_dbs(cookie_dbs: &Vec<CookieDB>) -> State {
        // The profiles list will never change after launch
        let profiles = StatefulList {
            status: ListState::default(),
            items: cookie_dbs.iter().map(|c| {
                c.path_short()
            }).collect()
        };
        State {
            selected_split: 0, 
            profiles, 
            current_domains: StatefulList::default(), 
            current_cookies: StatefulList::default(), 
            current_fields:  StatefulList::default(),
            cookie_dbs
        }
    }

    /// The currently selected domain (if any)
    pub fn selected_domain(&self) -> Option<String> {
        if let Some(selected_idx) = self.current_domains.status.selected() {
            // Convert to String to dodge BC
            Some(String::from(*self.current_domains.items.get(selected_idx).unwrap()))
        } else {
            None
        }
    }

    /// The currently selected cookie (if any)
    pub fn selected_cookie(&self) -> Option<String> {
        if let Some(selected_idx) = self.current_cookies.status.selected() {
            // Convert to String to dodge BC
            Some(String::from(*self.current_cookies.items.get(selected_idx).unwrap()))
        } else {
            None
        }
    }

}

