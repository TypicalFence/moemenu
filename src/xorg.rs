/*
 * This file is part of moemenu.
 * Copyright (C) 2021 fence.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */
use std::error::Error;
use std::{fmt, thread, time};

use x11rb::atom_manager;
use x11rb::connection::Connection;
use x11rb::errors::{ReplyError, ReplyOrIdError};
use x11rb::protocol::render::{self, ConnectionExt as _, PictType};
use x11rb::protocol::xproto::{ConnectionExt as _, *};
use x11rb::protocol::Event;
use x11rb::wrapper::ConnectionExt;
use x11rb::xcb_ffi::XCBConnection;

use crate::{Menu, Config, UserInterface, draw};
use crate::draw::{do_draw, set_color};
use crate::config::Position;

atom_manager! {
    pub AtomCollection: AtomCollectionCookie {
        WM_PROTOCOLS,
        WM_DELETE_WINDOW,
        _NET_WM_NAME,
        UTF8_STRING,
    }
}

pub struct XorgUserInterface {
    connection: XCBConnection,
    width: u16,
    height: u16,
    transparency: bool,
    window: Window,
    atoms: AtomCollection,
    surface: cairo::XCBSurface,
    config: Config
}

enum XorgUiAction {
    Redraw,
    Stop,
    Select,
    None,
}

#[allow(non_snake_case)]
mod XorgKeys {
    use x11rb::protocol::xproto::Keycode;

    pub const ESC: Keycode = 9;
    pub const ENTER: Keycode = 36;
    pub const BACKSPACE: Keycode = 22;
    pub const LEFT: Keycode = 113;
    pub const RIGHT: Keycode = 114;
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct xcb_visualtype_t {
    pub visual_id: u32,
    pub class: u8,
    pub bits_per_rgb_value: u8,
    pub colormap_entries: u16,
    pub red_mask: u32,
    pub green_mask: u32,
    pub blue_mask: u32,
    pub pad0: [u8; 4],
}

impl From<Visualtype> for xcb_visualtype_t {
    fn from(value: Visualtype) -> xcb_visualtype_t {
        xcb_visualtype_t {
            visual_id: value.visual_id,
            class: value.class.into(),
            bits_per_rgb_value: value.bits_per_rgb_value,
            colormap_entries: value.colormap_entries,
            red_mask: value.red_mask,
            green_mask: value.green_mask,
            blue_mask: value.blue_mask,
            pad0: [0; 4],
        }
    }
}

#[derive(Debug)]
struct KeyboardGrabError {
    details: String,
}

impl KeyboardGrabError {
    fn new(msg: &str) -> Self {
        KeyboardGrabError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for KeyboardGrabError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for KeyboardGrabError {
    fn description(&self) -> &str {
        &self.details
    }
}

#[derive(Debug)]
struct NoSelectionError {
}

impl NoSelectionError {
    fn new() -> Self {
        NoSelectionError {}
    }
}

impl fmt::Display for NoSelectionError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "No selection as made.")
    }
}

impl Error for NoSelectionError {
    fn description(&self) -> &str {
        "No selection as made."
    }
}

// wrapper around xorg.c
#[allow(non_upper_case_globals)]
mod sys {
    use std::os::raw::c_char;
    use x11rb::protocol::xproto::Keycode;
    
    //const XBufferOverflow: i8 = -1;
    //const XLookupNone: i8 = 1;
    const XLookupChars: i32 = 2;
    //const XLookupKeySym: i8 = 3;
    const XLookupBoth: i32 = 4;

    extern "C" {
        fn keycode_to_utf8(keycode: u32, mask: u32, buffer: *mut c_char) -> i32;
    }

    pub fn keycode_to_char(keycode: Keycode, state: u16) -> Option<char> {
        let mut buffer: [c_char; 32] = [0; 32];
        let status: i32 = unsafe {
            keycode_to_utf8(keycode as u32, state as u32, buffer.as_mut_ptr())
        };
        
        // if we received bytes try converting them into a char
        if status == XLookupChars || status == XLookupBoth {
            // why does rust define c_char as i8 ???
            let proper_bytes: Vec<u8> = buffer.to_vec().iter().map(|x| x.clone() as u8).collect();
            let str = std::str::from_utf8(proper_bytes.as_slice()).unwrap();
            let trimmed = str.trim_matches(char::from(0));
            return trimmed.chars().next()
        }
        
        None
    }

}

/// Find a `xcb_visualtype_t` based on its ID number
fn find_xcb_visualtype(conn: &impl Connection, visual_id: u32) -> Option<xcb_visualtype_t> {
    for root in &conn.setup().roots {
        for depth in &root.allowed_depths {
            for visual in &depth.visuals {
                if visual.visual_id == visual_id {
                    return Some((*visual).into());
                }
            }
        }
    }
    None
}

/// Choose a visual to use. This function tries to find a depth=32 visual and falls back to the
/// screen's default visual.
fn choose_visual(conn: &impl Connection, screen_num: usize) -> Result<(u8, Visualid), ReplyError> {
    let depth = 32;
    let screen = &conn.setup().roots[screen_num];

    // Try to use XRender to find a visual with alpha support
    let has_render = conn
        .extension_information(render::X11_EXTENSION_NAME)?
        .is_some();
    if has_render {
        let formats = conn.render_query_pict_formats()?.reply()?;
        // Find the ARGB32 format that must be supported.
        let format = formats
            .formats
            .iter()
            .filter(|info| (info.type_, info.depth) == (PictType::DIRECT, depth))
            .filter(|info| {
                let d = info.direct;
                (d.red_mask, d.green_mask, d.blue_mask, d.alpha_mask) == (0xff, 0xff, 0xff, 0xff)
            })
            .find(|info| {
                let d = info.direct;
                (d.red_shift, d.green_shift, d.blue_shift, d.alpha_shift) == (16, 8, 0, 24)
            });
        if let Some(format) = format {
            // Now we need to find the visual that corresponds to this format
            if let Some(visual) = formats.screens[screen_num]
                .depths
                .iter()
                .flat_map(|d| &d.visuals)
                .find(|v| v.format == format.id)
            {
                return Ok((format.depth, visual.visual));
            }
        }
    }
    Ok((screen.root_depth, screen.root_visual))
}

/// Check if a composite manager is running
fn composite_manager_running(
    conn: &impl Connection,
    screen_num: usize,
) -> Result<bool, ReplyError> {
    let atom = format!("_NET_WM_CM_S{}", screen_num);
    let atom = conn.intern_atom(false, atom.as_bytes())?.reply()?.atom;
    let owner = conn.get_selection_owner(atom)?.reply()?;
    Ok(owner.owner != x11rb::NONE)
}

/// returns the position and size of the focused monitor
#[cfg(feature = "multimonitor")]
fn handle_multi_monitor<C>(conn: &C, root: Window) -> Option<(i16, i16, u16, u16)> where
C: Connection,
{
    use x11rb::protocol::xinerama::{ConnectionExt as _, *};
    use x11rb::x11_utils::TryParse;
    use x11rb::cookie::Cookie;
    use x11rb::errors::{ConnectionError};

    // hecking generics (maybe define as macro?)
    fn unwrap_cookie<C, R>(cookie: Result<Cookie<C, R>, ConnectionError>) -> Option<R> where C: Connection, R: TryParse {
        if let Ok(cookie) = cookie {
            if let Ok(reply) = cookie.reply() {
                return Some(reply);
            }
        };
        None
    };

    let get_screen_info = |conn: &C| -> Option<QueryScreensReply> {
        unwrap_cookie(conn.xinerama_query_screens())
    };

    let get_focused_window = |conn: &C| -> Option<Window> {
        if let Some(reply) = unwrap_cookie(conn.get_input_focus()) {
            return Some(reply.focus);
        }
        None
    };

    let get_geometry= |conn: &C, w| -> Option<GetGeometryReply> {
        unwrap_cookie(conn.get_geometry(w))
    };

    let get_window_tree = |conn: &C, w| {
        unwrap_cookie(conn.query_tree(w))
    };

    let translate_coordinates = |conn: &C, src, dst| -> Option<TranslateCoordinatesReply> {
        if let Some(geo) = get_geometry(conn, src) {
            return unwrap_cookie(conn.translate_coordinates(src, dst, geo.x, geo.y));
        }
        None
    };

    let get_coords = |conn: &C,  w| -> Option<(i16, i16)> {
        if let Some(tree) = get_window_tree(&conn, w) {
            let root = tree.root;
            if let Some(translated) = translate_coordinates(conn, w, root) {
                return Some((translated.dst_x, translated.dst_y))
            }
        }
        None
    };

    let on_screen =  |x: i16, y: i16, screen: ScreenInfo| -> bool {
        if screen.x_org <= x && x <= screen.x_org + screen.width as i16 && screen.y_org <= y && y <= screen.y_org + screen.height as i16 {
            return true
        }

        false
    };

    let handle = |conn: &C| -> Option<(i16, i16, u16, u16)> {
        let point: Option<(i16, i16)> =(|| {
            // try to find focused monitor based on the focused window
            if let Some(window) = get_focused_window(conn) {
                return get_coords(conn, window)
                // try to base it on the pointer location instead
            } else if let Some(pointer) = unwrap_cookie(conn.query_pointer(root)) {
                return Some((pointer.root_x, pointer.root_y))
            }

            None
        })();

        if let Some((x, y)) = point {
            if let Some(screens) = get_screen_info(&conn) {
                for info in screens.screen_info {
                    if on_screen(x, y, info) {
                        return Some((info.x_org, info.y_org, info.width, info.height));
                    }
                }
            }
        }

        None
    };

    // TODO handle pointer position & -m argument
    handle(conn)
}

fn create_window<C>(
    conn: &C,
    screen: &Screen,
    atoms: &AtomCollection,
    height: u16,
    depth: u8,
    visual_id: Visualid,
    position: Position,
) -> Result<(Window, u16), ReplyOrIdError>
where
    C: Connection,
{
    let (screen_x, y_offset, screen_w, screen_h) = (|_conn: &C, screen: &Screen| {
        #[cfg(feature = "multimonitor")]
        if let Some(screen_pos) = handle_multi_monitor(_conn, screen.root) {
            return screen_pos;
        }

        (0, 0, screen.width_in_pixels, screen.height_in_pixels)
    })(conn, screen);
    let screen_y = match position {
        Position::Top => y_offset,
        Position::Bottom => y_offset + screen_h as i16 - height as i16
    };

    let window = conn.generate_id()?;
    let colormap = conn.generate_id()?;
    conn.create_colormap(ColormapAlloc::NONE, colormap, screen.root, visual_id)?;
    let win_aux = CreateWindowAux::new()
        .event_mask(EventMask::EXPOSURE | EventMask::STRUCTURE_NOTIFY)
        .background_pixel(x11rb::NONE)
        .border_pixel(screen.black_pixel)
        .colormap(colormap)
        .override_redirect(1);
    conn.create_window(
        depth,
        window,
        screen.root,
        screen_x,
        screen_y,
        screen_w,
        height,
        0,
        WindowClass::INPUT_OUTPUT,
        visual_id,
        &win_aux,
    )?;
    conn.free_colormap(colormap)?;

    let title = "moemenu";
    conn.change_property8(
        PropMode::REPLACE,
        window,
        AtomEnum::WM_NAME,
        AtomEnum::STRING,
        title.as_bytes(),
    )?;
    conn.change_property8(
        PropMode::REPLACE,
        window,
        atoms._NET_WM_NAME,
        atoms.UTF8_STRING,
        title.as_bytes(),
    )?;
    conn.change_property32(
        PropMode::REPLACE,
        window,
        atoms.WM_PROTOCOLS,
        AtomEnum::ATOM,
        &[atoms.WM_DELETE_WINDOW],
    )?;
    conn.change_property8(
        PropMode::REPLACE,
        window,
        AtomEnum::WM_CLASS,
        AtomEnum::STRING,
        b"moemenu\0moemenu\0",
    )?;

    conn.map_window(window)?;
    Ok((window, screen_w))
}

fn handle_keyboard(event: KeyPressEvent, menu: &mut Menu) -> XorgUiAction {
    // response_type 2 => press
    // response_type 3 => release
    match event {
        KeyPressEvent {
            response_type: 2,
            detail,
            state,
            ..
        } => match detail {
            XorgKeys::ENTER => XorgUiAction::Select,
            XorgKeys::ESC => XorgUiAction::Stop,
            XorgKeys::LEFT => {
                menu.select_previous_item();
                XorgUiAction::Redraw
            },
            XorgKeys::RIGHT =>  {
                menu.select_next_item();
                XorgUiAction::Redraw
            },
            XorgKeys::BACKSPACE => {
                let mut search_term= menu.get_search_term().clone();
                search_term.pop();
                menu.search(search_term);
                XorgUiAction::Redraw
            },
            key => handle_text(menu, key, state),
        },
        _ => {
            return XorgUiAction::None;
        }
    }
}

fn handle_text(menu: &mut Menu, key: Keycode, mask: u16) -> XorgUiAction {
    let char = sys::keycode_to_char(key, mask);

    if char.is_some() {
        let search_term = menu.get_search_term();
        menu.search(format!("{}{}", search_term, char.unwrap()));
        return XorgUiAction::Redraw;
    }

    XorgUiAction::None
}

fn grab_keyboard(conn: &XCBConnection, screen: &Screen) -> Result<(), KeyboardGrabError> {
    let wait_time = time::Duration::from_millis(10);
    let root = screen.root;
    for _ in 1..=100 {
        let cookie = conn
            .grab_keyboard(true, root, 0 as u32, GrabMode::ASYNC, GrabMode::ASYNC)
            .unwrap();

        if cookie.reply().unwrap().status == GrabStatus::SUCCESS {
            return Ok(());
        }
        
        thread::sleep(wait_time);
    }

    Err(KeyboardGrabError::new("failed grab keyboard"))
}

impl XorgUserInterface {
    pub fn new(config: Config) -> Result<XorgUserInterface, Box<dyn std::error::Error>> {
        let (conn, screen_num) = XCBConnection::connect(None)?;
        let screen = &conn.setup().roots[screen_num];
        let atoms = AtomCollection::new(&conn)?.reply()?;
        let height = config.height;
        let (depth, visualid) = choose_visual(&conn, screen_num)?;

        // Check if a composite manager is running. In a real application, we should also react to a
        // composite manager starting/stopping at runtime.
        let transparency = composite_manager_running(&conn, screen_num)?;

        // grab keyboard
        let grab_result = grab_keyboard(&conn, &screen);

        if grab_result.is_err() {
            return Err(Box::from(grab_result.err().unwrap()))
        }

        let (window, width) = create_window(&conn, &screen, &atoms, height, depth, visualid, config.position.clone())?;

        // Here comes all the interaction between cairo and x11rb:
        let mut visual = find_xcb_visualtype(&conn, visualid).unwrap();
        // SAFETY: cairo-rs just passes the pointer to C code and C code uses the xcb_connection_t, so
        // "nothing really" happens here, except that the borrow checked cannot check the lifetimes.
        let cairo_conn =
            unsafe { cairo::XCBConnection::from_raw_none(conn.get_raw_xcb_connection() as _) };
        let visual = unsafe { cairo::XCBVisualType::from_raw_none(&mut visual as *mut _ as _) };
        let surface = cairo::XCBSurface::create(
            &cairo_conn,
            &cairo::XCBDrawable(window),
            &visual,
            width.into(),
            height.into(),
        )
        .unwrap();

        // remove flicker by painting the window in the background color
        // not ideal but works
        let cr = cairo::Context::new(&surface);
        if transparency {
            cr.set_operator(cairo::Operator::Source);
        }
        set_color(&cr, config.colors.background);
        cr.paint();
        conn.flush()?;

        Ok(XorgUserInterface {
            connection: conn,
            window,
            surface,
            atoms,
            width,
            height,
            transparency,
            config,
        })
    }
}

impl UserInterface for XorgUserInterface {
    fn run(&mut self, menu: &mut Menu) -> Result<String, Box<dyn std::error::Error>> {
        let cr = cairo::Context::new(&self.surface);
        loop {
            self.connection.flush()?;
            let event = self.connection.wait_for_event()?;
            let mut event_option = Some(event);
            let mut need_redraw = false;
            while let Some(event) = event_option {
                match event {
                    Event::Expose(_) => {
                        need_redraw = true;
                    }
                    Event::ClientMessage(event) => {
                        let data = event.data.as_data32();
                        if event.format == 32
                            && event.window == self.window
                            && data[0] == self.atoms.WM_DELETE_WINDOW
                        {
                            eprintln!("Window was asked to close");
                            return Err(Box::from(NoSelectionError::new()));
                        }
                    }
                    Event::KeyPress(event) | Event::KeyRelease(event) => {
                        match handle_keyboard(event, menu) {
                            XorgUiAction::Stop => {
                                return Err(Box::from(NoSelectionError::new()));
                            }
                            XorgUiAction::Select => {
                                return match menu.get_selected_item() {
                                    Some(s) => Ok(s),
                                    None => Ok(menu.get_search_term()) 
                                }
                            }
                            XorgUiAction::Redraw => {
                                need_redraw = true;
                            }
                            XorgUiAction::None => {}
                        };
                    }
                    Event::Error(_) => eprintln!("Got an unexpected error"),
                    e => eprintln!("Got an unknown event: {:?}", e),
                }
                event_option = self.connection.poll_for_event()?;
            }

            // ensure selection does not go of screen
            let items = menu.get_items();
            let last_item = draw::find_last_item_that_fits(&cr, self.width as f64, menu.get_shift() as usize, items);
            let first_item = draw::find_first_item_that_fits(&cr, self.width as f64, menu.get_shift() as usize, items);
            menu.update_page(first_item, last_item);


            if need_redraw {
                do_draw(
                    &cr,
                    (self.width as _, self.height as _),
                    self.transparency,
                    &self.config,
                    &menu,
                );
                self.surface.flush();
            }
        }
    }
}
