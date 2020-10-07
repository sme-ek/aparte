/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;
use xmpp_parsers::bookmarks;
use xmpp_parsers::bookmarks2;
use xmpp_parsers::data_forms::{DataForm, DataFormType, Field, FieldType};
use xmpp_parsers::iq::{Iq, IqType};
use xmpp_parsers::ns;
use xmpp_parsers::pubsub::{owner as pubsubowner, PubSubOwner};
use xmpp_parsers::pubsub::{
    pubsub, pubsub::Items, pubsub::Publish, pubsub::PublishOptions, pubsub::Retract, Item, ItemId,
    NodeName, PubSub,
};
use xmpp_parsers::Element;
use xmpp_parsers::{BareJid, Jid};

use crate::command::{Command, CommandParser};
use crate::contact;
use crate::core::{Aparte, Event, Plugin};
use crate::plugins::disco;

command_def!(bookmark_add,
r#"/bookmark add <bookmark> <conference> [autojoin=on|off]

    bookmark    The bookmark friendly name
    conference  The conference room jid
    autojoin    Wether the conference room should be automatically joined on startup

Description:
    Add a bookmark

Examples:
    /bookmark add aparte aparte@conference.fariello.eu
    /bookmark add aparte aparte@conference.fariello.eu/mynick
    /bookmark add aparte aparte@conference.fariello.eu/mynick autojoin=on
"#,
{
    name: String,
    conference: Jid,
    autojoin: Option<bool>
},
|aparte, _command| {
    let add = {
        let mut bookmarks = aparte.get_plugin_mut::<BookmarksPlugin>().unwrap();
        let nick = match conference.clone() {
            Jid::Bare(_room) => None,
            Jid::Full(room) => Some(room.resource),
        };
        let autojoin = match autojoin {
            None => false,
            Some(autojoin) => autojoin,
        };
        let bookmark = contact::Bookmark {
            jid: conference.into(),
            name: Some(name),
            nick: nick,
            password: None,
            autojoin: autojoin,
        };
        bookmarks.add(bookmark)
    };
    aparte.send(add);
    Ok(())
});

command_def!(
    bookmark_del,
    r#"/bookmark del <bookmark>

    bookmark    The bookmark friendly name

Description:
    Delete a bookmark

Examples:
    /bookmark del aparte
"#,
    { conference: Jid },
    |aparte, _command| {
        let delete = {
            let mut bookmarks = aparte.get_plugin_mut::<BookmarksPlugin>().unwrap();
            bookmarks.delete(conference)
        };
        aparte.send(delete);
        Ok(())
    }
);

command_def!(bookmark_edit,
r#"/bookmark edit <bookmark> [<conference>] [autojoin=on|off]

    bookmark    The bookmark friendly name
    conference  The conference room jid
    autojoin    Wether the conference room should be automatically joined on startup

Description:
    Edit a bookmark

Examples:
    /bookmark edit aparte autojoin=on
    /bookmark edit aparte aparte@conference.fariello.eu
    /bookmark edit aparte aparte@conference.fariello.eu autojoin=off
"#,
{
    name: String,
    conference: Jid,
    autojoin: Option<bool>
},
|aparte, _command| {
    // TODO download bookmark first to keep extensions elements
    let add = {
        let mut bookmarks = aparte.get_plugin_mut::<BookmarksPlugin>().unwrap();
        let nick = match conference.clone() {
            Jid::Bare(_room) => None,
            Jid::Full(room) => Some(room.resource),
        };
        let autojoin = match autojoin {
            None => false,
            Some(autojoin) => autojoin,
        };
        let bookmark = contact::Bookmark {
            jid: conference.into(),
            name: Some(name),
            nick: nick,
            password: None,
            autojoin: autojoin,
        };
        bookmarks.add(bookmark)
    };
    aparte.send(add);
    Ok(())
});

command_def!(bookmark,
r#"/bookmark add|del|edit"#,
{
    action: Command = {
        children: {
            "add": bookmark_add,
            "del": bookmark_del,
            "edit": bookmark_edit,
        }
    },
});

enum Backend {
    Bookmarks(Bookmarks),
    Bookmarks2(Bookmarks2),
}

struct Bookmarks {}

impl Bookmarks {
    fn retreive(&self) -> Element {
        let id = Uuid::new_v4().to_hyphenated().to_string();
        let items = Items {
            max_items: None,
            node: NodeName(String::from(ns::BOOKMARKS)),
            subid: None,
            items: vec![],
        };
        let pubsub = PubSub::Items(items);
        let iq = Iq::from_get(id, pubsub);
        iq.into()
    }

    fn create_node(&self) -> Element {
        todo!();
    }

    fn config_node(&self) -> Element {
        todo!();
    }

    fn add(&self, bookmark: contact::Bookmark) -> Element {
        todo!();
    }

    fn delete(&self, conference: Jid) -> Element {
        todo!();
    }

    fn handle(&self, items: pubsub::Items) -> Vec<contact::Bookmark> {
        let mut bookmarks = vec![];
        for item in items.items {
            if let Some(el) = item.payload.clone() {
                if let Ok(storage) = bookmarks::Storage::try_from(el) {
                    for conf in storage.conferences {
                        let bookmark = contact::Bookmark {
                            jid: conf.jid.clone(),
                            name: Some(conf.name.clone()),
                            nick: conf.nick.clone(),
                            password: conf.password.clone(),
                            autojoin: conf.autojoin == bookmarks::Autojoin::True,
                        };

                        bookmarks.push(bookmark);
                    }
                }
            } else {
                warn!("Missing storage element");
            }
        }

        bookmarks
    }

    fn init(&self) -> Vec<Element> {
        let elems = vec![];
        elems
    }
}

struct Bookmarks2 {}

impl Bookmarks2 {
    pub fn retreive(&self) -> Element {
        let id = Uuid::new_v4().to_hyphenated().to_string();
        let items = Items {
            max_items: None,
            node: NodeName(String::from(ns::BOOKMARKS2)),
            subid: None,
            items: vec![],
        };
        let pubsub = PubSub::Items(items);
        let iq = Iq::from_get(id, pubsub);
        iq.into()
    }

    fn config_node_form(&self) -> DataForm {
        DataForm {
            type_: DataFormType::Submit,
            form_type: Some(String::from(
                "http://jabber.org/protocol/pubsub#node_config",
            )),
            title: None,
            instructions: None,
            fields: vec![
                Field {
                    var: String::from("pubsub#persist_items"),
                    type_: FieldType::Boolean,
                    label: None,
                    required: false,
                    media: vec![],
                    options: vec![],
                    values: vec![String::from("true")],
                },
                Field {
                    var: String::from("pubsub#send_last_published_item"),
                    type_: FieldType::TextSingle,
                    label: None,
                    required: false,
                    media: vec![],
                    options: vec![],
                    values: vec![String::from("never")],
                },
                Field {
                    var: String::from("pubsub#access_model"),
                    type_: FieldType::TextSingle,
                    label: None,
                    required: false,
                    media: vec![],
                    options: vec![],
                    values: vec![String::from("whitelist")],
                },
                Field {
                    var: String::from("pubsub#max_items"),
                    type_: FieldType::TextSingle,
                    label: None,
                    required: false,
                    media: vec![],
                    options: vec![],
                    values: vec![String::from("10")],
                },
            ],
        }
    }

    fn create_node(&self) -> Element {
        let id = Uuid::new_v4().to_hyphenated().to_string();
        let create = pubsub::Create {
            node: Some(NodeName(String::from(ns::BOOKMARKS2))),
        };
        let pubsub = PubSub::Create {
            create: create,
            configure: None,
        };
        let iq = Iq::from_set(id, pubsub);
        iq.into()
    }

    fn config_node(&self) -> Element {
        let id = Uuid::new_v4().to_hyphenated().to_string();
        let config = pubsubowner::Configure {
            node: Some(NodeName(String::from(ns::BOOKMARKS2))),
            form: Some(self.config_node_form()),
        };
        let pubsub = PubSubOwner::Configure(config);
        let iq = Iq::from_set(id, pubsub);
        iq.into()
    }

    fn add(&self, bookmark: contact::Bookmark) -> Element {
        let id = Uuid::new_v4().to_hyphenated().to_string();
        let item = Item {
            id: Some(ItemId(bookmark.jid.to_string())),
            payload: Some(
                bookmarks2::Conference {
                    autojoin: match bookmark.autojoin {
                        true => bookmarks2::Autojoin::True,
                        false => bookmarks2::Autojoin::False,
                    },
                    name: bookmark.name,
                    nick: bookmark.nick,
                    password: None,
                }
                .into(),
            ),
            publisher: None,
        };
        let publish = Publish {
            node: NodeName(String::from(ns::BOOKMARKS2)),
            items: vec![pubsub::Item(item)],
        };
        let options = PublishOptions {
            form: Some(DataForm {
                type_: DataFormType::Submit,
                form_type: Some(String::from(
                    "http://jabber.org/protocol/pubsub#publish-options",
                )),
                title: None,
                instructions: None,
                fields: vec![
                    Field {
                        var: String::from("pubsub#persist_items"),
                        type_: FieldType::Boolean,
                        label: None,
                        required: false,
                        media: vec![],
                        options: vec![],
                        values: vec![String::from("true")],
                    },
                    Field {
                        var: String::from("pubsub#access_model"),
                        type_: FieldType::TextSingle,
                        label: None,
                        required: false,
                        media: vec![],
                        options: vec![],
                        values: vec![String::from("whitelist")],
                    },
                ],
            }),
        };
        let pubsub = PubSub::Publish {
            publish: publish,
            publish_options: Some(options),
        };
        let iq = Iq::from_set(id, pubsub);
        iq.into()
    }

    fn delete(&self, conference: Jid) -> Element {
        let id = Uuid::new_v4().to_hyphenated().to_string();
        let item = Item {
            id: Some(ItemId(conference.into())),
            payload: None,
            publisher: None,
        };
        let retract = Retract {
            node: NodeName(String::from(ns::BOOKMARKS2)),
            items: vec![pubsub::Item(item)],
            notify: pubsub::Notify::False,
        };
        let pubsub = PubSub::Retract(retract);
        let iq = Iq::from_set(id, pubsub);
        iq.into()
    }

    fn handle(&self, items: pubsub::Items) -> Vec<contact::Bookmark> {
        let mut bookmarks = vec![];
        for item in items.items {
            if let Some(id) = item.id.clone() {
                if let Ok(bare_jid) = BareJid::from_str(&id.0) {
                    if let Some(el) = item.payload.clone() {
                        if let Ok(conf) = bookmarks2::Conference::try_from(el) {
                            let bookmark = contact::Bookmark {
                                jid: bare_jid.clone(),
                                name: conf.name.clone(),
                                nick: conf.nick.clone(),
                                password: conf.password.clone(),
                                autojoin: conf.autojoin == bookmarks2::Autojoin::True,
                            };

                            bookmarks.push(bookmark);
                        }
                    } else {
                        warn!("Empty bookmark element {}", id.0);
                    }
                } else {
                    warn!("Invalid bookmark jid {}", id.0);
                }
            } else {
                warn!("Missing bookmark id");
            }
        }

        bookmarks
    }

    fn init(&self) -> Vec<Element> {
        let mut elems = vec![];
        elems.push(self.create_node());
        elems.push(self.config_node());
        elems
    }
}

pub struct BookmarksPlugin {
    backend: Backend,
    pub bookmarks: Vec<contact::Bookmark>,
    pub bookmarks_by_name: HashMap<String, usize>,
    pub bookmarks_by_jid: HashMap<Jid, usize>,
}

impl BookmarksPlugin {
    fn retreive(&self) -> Element {
        match &self.backend {
            Backend::Bookmarks(backend) => backend.retreive(),
            Backend::Bookmarks2(backend) => backend.retreive(),
        }
    }

    fn init_backend(&self) -> Vec<Element> {
        match &self.backend {
            Backend::Bookmarks(backend) => backend.init(),
            Backend::Bookmarks2(backend) => backend.init(),
        }
    }

    fn add(&mut self, bookmark: contact::Bookmark) -> Element {
        self.bookmarks.push(bookmark.clone());

        match &self.backend {
            Backend::Bookmarks(backend) => backend.add(bookmark),
            Backend::Bookmarks2(backend) => backend.add(bookmark),
        }
    }

    fn delete(&mut self, conference: Jid) -> Element {
        // TODO remove from local storage

        match &self.backend {
            Backend::Bookmarks(backend) => backend.delete(conference),
            Backend::Bookmarks2(backend) => backend.delete(conference),
        }
    }

    fn handle_bookmarks(&mut self, aparte: &mut Aparte, items: pubsub::Items) {
        self.bookmarks = match (&items.node.0 as &str, &self.backend) {
            (ns::BOOKMARKS, Backend::Bookmarks(backend)) => backend.handle(items.clone()),
            (ns::BOOKMARKS2, Backend::Bookmarks2(backend)) => backend.handle(items.clone()),
            _ => return,
        };

        self.bookmarks_by_name = self
            .bookmarks
            .iter()
            .enumerate()
            .filter(|(_, bookmark)| bookmark.name.is_some())
            .map(|(index, bookmark)| (bookmark.name.clone().unwrap(), index))
            .collect();
        self.bookmarks_by_jid = self
            .bookmarks
            .iter()
            .enumerate()
            .map(|(index, bookmark)| (bookmark.jid.clone().into(), index))
            .collect();

        for bookmark in self.bookmarks.iter() {
            aparte.schedule(Event::Bookmark(bookmark.clone()));
            if bookmark.autojoin {
                let jid = match &bookmark.nick {
                    Some(nick) => Jid::Full(bookmark.jid.clone().with_resource(nick)),
                    None => Jid::Bare(bookmark.jid.clone()),
                };
                info!("Autojoin {}", jid.to_string());
                aparte.schedule(Event::Join(jid, false));
            }
        }
    }

    pub fn get_by_name(&self, name: &str) -> Option<contact::Bookmark> {
        match self.bookmarks_by_name.get(name) {
            Some(index) => self.bookmarks.get(*index).cloned(),
            None => None,
        }
    }
}

impl Plugin for BookmarksPlugin {
    fn new() -> BookmarksPlugin {
        BookmarksPlugin {
            backend: Backend::Bookmarks(Bookmarks {}),
            bookmarks: vec![],
            bookmarks_by_name: HashMap::new(),
            bookmarks_by_jid: HashMap::new(),
        }
    }

    fn init(&mut self, aparte: &mut Aparte) -> Result<(), ()> {
        aparte.add_command(bookmark::new());
        let mut disco = aparte.get_plugin_mut::<disco::Disco>().unwrap();
        disco.add_feature(ns::BOOKMARKS2)
    }

    fn on_event(&mut self, aparte: &mut Aparte, event: &Event) {
        match event {
            Event::Disco => {
                {
                    let disco = aparte.get_plugin::<disco::Disco>().unwrap();
                    if disco.has_feature(ns::BOOKMARKS2) {
                        self.backend = Backend::Bookmarks2(Bookmarks2 {});
                    }
                }

                for elem in self.init_backend().drain(..) {
                    aparte.send(elem);
                }
                aparte.send(self.retreive());
            }
            Event::Iq(iq) => match iq.payload.clone() {
                IqType::Result(Some(el)) => {
                    if let Ok(PubSub::Items(items)) = PubSub::try_from(el) {
                        match &items.node.0 as &str {
                            ns::BOOKMARKS | ns::BOOKMARKS2 => {
                                self.handle_bookmarks(aparte, items.clone())
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}

impl fmt::Display for BookmarksPlugin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "XEP-0402: PEP Native Bookmarks")
    }
}
