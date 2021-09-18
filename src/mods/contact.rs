/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;

use uuid::Uuid;
use xmpp_parsers::iq::{Iq, IqType};
use xmpp_parsers::{ns, presence, roster, BareJid, Element, Jid};

use crate::account::Account;
use crate::contact;
use crate::core::{Aparte, Event, ModTrait};

impl From<roster::Group> for contact::Group {
    fn from(item: roster::Group) -> Self {
        Self(item.0)
    }
}

impl From<roster::Item> for contact::Contact {
    fn from(item: roster::Item) -> Self {
        let mut groups = Vec::new();
        for group in item.groups {
            groups.push(group.into());
        }

        Self {
            jid: item.jid.clone(),
            name: item.name.clone(),
            subscription: item.subscription.clone(),
            presence: contact::Presence::Unavailable,
            groups: groups,
        }
    }
}

#[derive(Eq, PartialEq, Hash)]
pub struct ContactIndex {
    account: Account,
    jid: BareJid,
}

pub struct ContactMod {
    pub contacts: HashMap<ContactIndex, contact::Contact>,
}

impl ContactMod {
    pub fn new() -> Self {
        Self {
            contacts: HashMap::new(),
        }
    }

    fn request(&self) -> Element {
        let id = Uuid::new_v4().to_hyphenated().to_string();
        let iq = Iq::from_get(
            id,
            roster::Roster {
                ver: None,
                items: Vec::new(),
            },
        );
        iq.into()
    }
}

impl ModTrait for ContactMod {
    fn init(&mut self, _aparte: &mut Aparte) -> Result<(), ()> {
        Ok(())
    }

    fn on_event(&mut self, aparte: &mut Aparte, event: &Event) {
        match event {
            Event::Connected(account, _jid) => aparte.send(account, self.request()),
            Event::Iq(account, iq) => {
                if let IqType::Result(Some(payload)) = iq.payload.clone() {
                    if payload.is("query", ns::ROSTER) {
                        if let Ok(roster) = roster::Roster::try_from(payload.clone()) {
                            for item in roster.items {
                                let contact: contact::Contact = item.clone().into();
                                let index = ContactIndex {
                                    account: account.clone(),
                                    jid: contact.jid.clone(),
                                };
                                self.contacts.insert(index, contact.clone());
                                aparte.schedule(Event::Contact(account.clone(), contact.clone()));
                            }
                        }
                    }
                }
            }
            Event::Presence(account, presence) => {
                if let Some(from) = &presence.from {
                    let jid = match from {
                        Jid::Bare(jid) => jid.clone(),
                        Jid::Full(jid) => jid.clone().into(),
                    };
                    let index = ContactIndex {
                        account: account.clone(),
                        jid,
                    };
                    if let Some(contact) = self.contacts.get_mut(&index) {
                        contact.presence = match presence.show {
                            Some(presence::Show::Away) => contact::Presence::Away,
                            Some(presence::Show::Chat) => contact::Presence::Chat,
                            Some(presence::Show::Dnd) => contact::Presence::Dnd,
                            Some(presence::Show::Xa) => contact::Presence::Xa,
                            None => contact::Presence::Available,
                        };
                        aparte.schedule(Event::ContactUpdate(account.clone(), contact.clone()));
                    }
                }
            }
            _ => {}
        }
    }
}

impl fmt::Display for ContactMod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Contact management")
    }
}
