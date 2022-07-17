use std::collections::HashMap;

use tui::widgets::ListState;

use crate::{
    config::NO_SELECTION,
    cookie_db::CookieDB,
    cookie::Cookie
};

pub struct StatefulList<T> {
    pub status: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn default() -> Self {
        StatefulList { status: ListState::default(), items: vec![] }
    }

    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            status: ListState::default(),
            items,
        }
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
pub struct State<'a> {
    pub selected_split: u32, // 0 - 3

    // We we only keep the domains for the currently seleceted profile
    // in a StatefulList. If a domain is removed, we will update the
    // underlying CookieDB and reload
    pub cookie_dbs: &'a Vec<CookieDB>,

    pub profiles:        StatefulList<String>,
    pub current_domains: StatefulList<&'a str>,
    pub current_cookies: StatefulList<&'a str>,
    pub current_fields:  StatefulList<&'a str>
}

impl<'a> State<'a> {
    /// Create a TUI state object from a vector of cookie databases
    pub fn from_cookie_dbs(cookie_dbs: &Vec<CookieDB>) -> State {

        // The profiles list will never change
        let profiles = StatefulList::with_items(

            cookie_dbs.iter().map(|c| {
                c.path_short()
            }).collect()
        );
        State {
            selected_split: 0, 
            profiles, 
            current_domains: StatefulList::default(), 
            current_cookies: StatefulList::default(), 
            current_fields:  StatefulList::default(),
            cookie_dbs
        }
    }


    pub fn selected_domain(&self) -> Option<&&str> {
        if let Some(selected_idx) = self.current_domains.status.selected() {
            self.current_domains.items.get(selected_idx)
        } else {
            None
        }

    }
}

