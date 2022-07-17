use std::collections::HashMap;

use tui::widgets::ListState;

use crate::{
    config::NO_SELECTION,
    cookie_db::CookieDB,
    cookie::Cookie
};

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
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

    pub fn previous(&mut self) {
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
    pub selected_split: u32,
    pub profiles: StatefulList<String>,

    // The cookies and domain lists will need to be updatable
    // profile              -> stateful list of domains
    // profile+domain       -> stateful list of cookies
    pub domains: HashMap<String,StatefulList<&'a str>>,
    pub cookies: HashMap<String,StatefulList<&'a Cookie>>,

    pub current_fields: StatefulList<&'a str>
}
impl<'a> State<'a> {
    /// Create a TUI state object from a vector of cookie databases
    pub fn from_cookie_dbs(cookie_dbs: &Vec<CookieDB>) -> State {
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
            selected_split: 0, profiles, domains, cookies, 
            current_fields: StatefulList { 
                state: ListState::default(), 
                items: vec![] 
            }
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
    /// as a mutable reference.
    /// The `render_stateful_widget()` method on a frame requires a mutable
    /// reference.
    pub fn domains_for_profile(&mut self) -> Option<&mut StatefulList<&'a str>> {
        let selected: usize = self.profiles.state.selected()
                .unwrap_or_else(|| NO_SELECTION);
        if selected != NO_SELECTION {
            // Note that the reference is mutable, this is required to call
            // e.g. `select()`
            self.domains.get_mut(self.profiles.items.get(selected).unwrap())
        } else {
           None
        }
    }

    /// Fetch the `StatefulList` of cookies for the currently selected domain
    /// of the current profile as a mutable reference.
    pub fn cookies_for_domain(&mut self) -> Option<&mut StatefulList<&'a Cookie>> {
        if let Some(current_profile) = self._selected_profile() {
           // .clone() to dodge BC
           let current_profile = current_profile.clone();

           if let Some(current_domains) = self.domains_for_profile() {

               let selected_idx = current_domains.state.selected()
                   .unwrap_or_else(|| NO_SELECTION);

               if selected_idx != NO_SELECTION {
                   let current_domain = // .clone() to dodge BC
                       current_domains.items.get(selected_idx).unwrap().clone();

                   let key = format!("{}{}", current_profile, current_domain);
                   return self.cookies.get_mut(&key);
               }
           }
        }
        None
    }

    /// Return the Cookie object for the currently selected `Cookie` (if any)
    pub fn fields_for_cookie(&mut self) -> Option<&'a Cookie> {
        if let Some(current_cookies) = self.cookies_for_domain() {
           let selected_idx = current_cookies.state.selected()
               .unwrap_or_else(|| NO_SELECTION);

           if selected_idx != NO_SELECTION {
                // .clone() to dodge BC
                return Some(current_cookies.items.get(selected_idx).unwrap().clone());
           }
        }
        None
    }
}

