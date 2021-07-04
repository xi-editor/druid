// Copyright 2020 The Druid Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Interactions with the system pasteboard on X11.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::Rc;

use x11rb::connection::Connection;
use x11rb::errors::{ConnectionError, ReplyOrIdError};
use x11rb::protocol::xproto::{
    AtomEnum, ChangeWindowAttributesAux, ConnectionExt, EventMask, GetPropertyReply,
    GetPropertyType, Property, PropertyNotifyEvent, SelectionClearEvent, SelectionRequestEvent,
    Timestamp, WindowClass,
};
use x11rb::protocol::Event;
use x11rb::xcb_ffi::XCBConnection;

use crate::clipboard::{ClipboardFormat, FormatId};
use tracing::{debug, warn};

// We can pick an arbitrary atom that is used for the transfer. This is our pick.
const TRANSFER_ATOM: AtomEnum = AtomEnum::CUT_BUFFE_R4;

const STRING_TARGETS: [&str; 5] = [
    "UTF8_STRING",
    "TEXT",
    "STRING",
    "text/plain;charset=utf-8",
    "text/plain",
];

x11rb::atom_manager! {
    ClipboardAtoms: ClipboardAtomsCookie {
        CLIPBOARD,
        TARGETS,
        INCR,
    }
}

#[derive(Debug, Clone)]
pub struct Clipboard(Rc<RefCell<ClipboardState>>);

impl Clipboard{
    pub(crate) fn new(
        connection: Rc<XCBConnection>,
        screen_num: usize,
        event_queue: Rc<RefCell<VecDeque<Event>>>,
        timestamp: Rc<Cell<Timestamp>>,
    ) -> Result<Self, ReplyOrIdError> {
        Ok(Self(Rc::new(RefCell::new(ClipboardState::new(connection, screen_num, event_queue, timestamp)?))))
    }

    pub(crate) fn handle_clear(&self, event: &SelectionClearEvent) -> Result<(), ConnectionError> {
        self.0.borrow_mut().handle_clear(event)
    }

    pub(crate) fn handle_request(&self, event: &SelectionRequestEvent) -> Result<(), ReplyOrIdError> {
        self.0.borrow_mut().handle_request(event)
    }

    pub(crate) fn handle_property_notify(&self, event: &PropertyNotifyEvent) -> Result<(), ReplyOrIdError> {
        self.0.borrow_mut().handle_property_notify(event)
    }

    pub fn put_string(&mut self, s: impl AsRef<str>) {
        self.0.borrow_mut().put_string(s.as_ref())
    }

    pub fn put_formats(&mut self, formats: &[ClipboardFormat]) {
        self.0.borrow_mut().put_formats(formats)
    }

    pub fn get_string(&self) -> Option<String> {
        self.0.borrow().get_string()
    }

    pub fn preferred_format(&self, formats: &[FormatId]) -> Option<FormatId> {
        self.0.borrow().preferred_format(formats)
    }

    pub fn get_format(&self, format: FormatId) -> Option<Vec<u8>> {
        self.0.borrow().get_format(format)
    }

    pub fn available_type_names(&self) -> Vec<String> {
        self.0.borrow().available_type_names()
    }
}

#[derive(Debug, Clone)]
struct ClipboardState {
    connection: Rc<XCBConnection>,
    screen_num: usize,
    atoms: ClipboardAtoms,
    event_queue: Rc<RefCell<VecDeque<Event>>>,
    timestamp: Rc<Cell<Timestamp>>,
}

impl ClipboardState {
    fn new(
        connection: Rc<XCBConnection>,
        screen_num: usize,
        event_queue: Rc<RefCell<VecDeque<Event>>>,
        timestamp: Rc<Cell<Timestamp>>,
    ) -> Result<Self, ReplyOrIdError> {
        let atoms = ClipboardAtoms::new(&*connection)?.reply()?;
        Ok(Self {
            connection,
            screen_num,
            atoms,
            event_queue,
            timestamp,
        })
    }

    fn put_string(&mut self, s: &str) {
        let formats = STRING_TARGETS.iter()
            .map(|format| ClipboardFormat::new(format, s.as_bytes()))
            .collect::<Vec<_>>();
        self.put_formats(&formats);
    }

    fn put_formats(&mut self, _formats: &[ClipboardFormat]) {
        // TODO(x11/clipboard): implement Clipboard::put_formats
        warn!("Clipboard::put_formats is currently unimplemented for X11 platforms.");
    }

    fn get_string(&self) -> Option<String> {
        STRING_TARGETS.iter().find_map(|target| {
            self.get_format(target)
                .and_then(|data| String::from_utf8(data).ok())
        })
    }

    fn preferred_format(&self, formats: &[FormatId]) -> Option<FormatId> {
        let available = self.available_type_names();
        formats
            .iter()
            .find(|f1| available.iter().any(|f2| *f1 == f2))
            .copied()
    }

    fn get_format(&self, format: FormatId) -> Option<Vec<u8>> {
        self.do_transfer(format, |prop| prop.value)
    }

    #[allow(clippy::needless_collect)]
    fn available_type_names(&self) -> Vec<String> {
        let requests = self
            .do_transfer("TARGETS", |prop| {
                prop.value32()
                    .map(|iter| iter.collect())
                    .unwrap_or_default()
            })
            .unwrap_or_default()
            .into_iter()
            .filter_map(|atom| self.connection.get_atom_name(atom).ok())
            .collect::<Vec<_>>();
        // We first send all requests above and then fetch the replies with only one round-trip to
        // the X11 server. Hence, the collect() above is not unnecessary!
        requests
            .into_iter()
            .filter_map(|req| req.reply().ok())
            .filter_map(|reply| String::from_utf8(reply.name).ok())
            .collect()
    }

    fn do_transfer<R, F>(&self, format: FormatId, converter: F) -> Option<Vec<R>>
    where
        R: Clone,
        F: FnMut(GetPropertyReply) -> Vec<R>,
    {
        match self.do_transfer_impl(format, converter) {
            Ok(result) => result,
            Err(error) => {
                warn!("Error in Clipboard::do_transfer: {:?}", error);
                None
            }
        }
    }

    fn do_transfer_impl<R, F>(
        &self,
        format: FormatId,
        mut converter: F,
    ) -> Result<Option<Vec<R>>, ReplyOrIdError>
    where
        R: Clone,
        F: FnMut(GetPropertyReply) -> Vec<R>,
    {
        debug!("Getting clipboard contents in format {}", format);

        let conn = &*self.connection;
        let format_atom = conn.intern_atom(false, format.as_bytes())?.reply()?.atom;

        // Create a window for the transfer
        let window = WindowContainer::new(conn, self.screen_num)?;

        conn.convert_selection(
            window.window,
            self.atoms.CLIPBOARD,
            format_atom,
            TRANSFER_ATOM,
            self.timestamp.get(),
        )?;

        // Now wait for the selection notify event
        conn.flush()?;
        let notify = loop {
            match conn.wait_for_event()? {
                Event::SelectionNotify(notify) if notify.requestor == window.window => {
                    break notify
                }
                event => self.event_queue.borrow_mut().push_back(event),
            }
        };

        if notify.property == x11rb::NONE {
            // Selection is empty
            debug!("Selection transfer was rejected");
            return Ok(None);
        }

        conn.change_window_attributes(
            window.window,
            &ChangeWindowAttributesAux::default().event_mask(EventMask::PROPERTY_CHANGE),
        )?;

        let property = conn
            .get_property(
                true,
                window.window,
                TRANSFER_ATOM,
                GetPropertyType::ANY,
                0,
                u32::MAX,
            )?
            .reply()?;

        if property.type_ != self.atoms.INCR {
            debug!("Got selection contents directly");
            return Ok(Some(converter(property)));
        }

        // The above GetProperty with delete=true indicated that the INCR transfer starts
        // now, wait for the property notifies
        debug!("Doing an INCR transfer for the selection");
        conn.flush()?;
        let mut value = Vec::new();
        loop {
            match conn.wait_for_event()? {
                Event::PropertyNotify(notify)
                    if (notify.window, notify.state) == (window.window, Property::NEW_VALUE) =>
                {
                    let property = conn
                        .get_property(
                            true,
                            window.window,
                            TRANSFER_ATOM,
                            GetPropertyType::ANY,
                            0,
                            u32::MAX,
                        )?
                        .reply()?;
                    if property.value.is_empty() {
                        debug!("INCR transfer finished");
                        return Ok(Some(value));
                    } else {
                        value.extend_from_slice(&converter(property));
                    }
                }
                event => self.event_queue.borrow_mut().push_back(event),
            }
        }
    }

    fn handle_clear(&self, _event: &SelectionClearEvent) -> Result<(), ConnectionError> {
        Ok(())
    }

    fn handle_request(&self, _event: &SelectionRequestEvent) -> Result<(), ReplyOrIdError> {
        Ok(())
    }

    fn handle_property_notify(&self, _event: &PropertyNotifyEvent) -> Result<(), ReplyOrIdError> {
        Ok(())
    }

}

struct WindowContainer<'a> {
    window: u32,
    conn: &'a XCBConnection,
}

impl<'a> WindowContainer<'a> {
    fn new(conn: &'a XCBConnection, screen_num: usize) -> Result<Self, ReplyOrIdError> {
        let window = conn.generate_id()?;
        conn.create_window(
            x11rb::COPY_DEPTH_FROM_PARENT,
            window,
            conn.setup().roots[screen_num].root,
            0,
            0,
            1,
            1,
            0,
            WindowClass::INPUT_OUTPUT,
            x11rb::COPY_FROM_PARENT,
            &Default::default(),
        )?;
        Ok(WindowContainer { window, conn })
    }
}

impl Drop for WindowContainer<'_> {
    fn drop(&mut self) {
        let _ = self.conn.destroy_window(self.window);
    }
}
