use std::error::Error;
use std::fmt;
use std::os::raw::c_char;

use x11::xlib;
use x11rb::atom_manager;
use x11rb::connection::Connection;
use x11rb::errors::{ReplyError, ReplyOrIdError};
use x11rb::protocol::render::{self, ConnectionExt as _, PictType};
use x11rb::protocol::xproto::{ConnectionExt as _, *};
use x11rb::protocol::Event;
use x11rb::wrapper::ConnectionExt;
use x11rb::xcb_ffi::XCBConnection;

use crate::draw::do_draw;
use crate::Menu;
use crate::UserInterface;
use std::ffi::CString;
use std::ptr::{null, null_mut};

// quite a lot of this was taken from the x11rb examples
// it would be kinda cool the unsafe bits could be removed at some point

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
    screen: usize,
    width: u16,
    height: u16,
    transparency: bool,
    window: Window,
    atoms: AtomCollection,
    surface: cairo::XCBSurface,
}

enum XorgUiAction {
    Redraw,
    Stop,
    None,
}

mod XorgKeys {
    use x11rb::protocol::xproto::Keycode;

    pub const ESC: Keycode = 9;
    pub const ENTER: Keycode = 36;
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

fn create_window<C>(
    conn: &C,
    screen: &x11rb::protocol::xproto::Screen,
    atoms: &AtomCollection,
    (width, height): (u16, u16),
    depth: u8,
    visual_id: Visualid,
) -> Result<Window, ReplyOrIdError>
where
    C: Connection,
{
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
        0,
        (screen.height_in_pixels - height) as i16,
        width,
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
    Ok(window)
}

fn handle_keyboard(event: KeyPressEvent, menu: &mut Menu) -> XorgUiAction {
    // response_type 2 => press
    // response_type 3 => release

    match event {
        KeyPressEvent {
            response_type: 3,
            detail,
            ..
        } => match detail {
            XorgKeys::ENTER => XorgUiAction::Stop,
            XorgKeys::ESC => XorgUiAction::Stop,
            key => handle_text(menu, key),
        },
        _ => {
            return XorgUiAction::None;
        }
    }
}

fn handle_text(menu: &mut Menu, key: Keycode) -> XorgUiAction {
    //get_string_from_keycode(key);
    XorgUiAction::None
}

fn get_string_from_keycode(key: Keycode) {
    // todo recreate in C and wrap that in rust
    // the x11 bindings are unusable

    unsafe {
        let display = xlib::XOpenDisplay(null_mut());
        let mut event = xlib::XKeyEvent {
            type_: 3,
            display,
            state: 0,
            keycode: key as u32,
            // random garbage
            // wish I could just do a partial struct
            serial: 0,
            send_event: 0,
            window: 0,
            root: 0,
            subwindow: 0,
            time: 0,
            x: 0,
            y: 0,
            x_root: 0,
            y_root: 0,
            same_screen: 1,
        };
        let im = xlib::XOpenIM(
            display,
            xlib::XrmGetDatabase(display),
            null_mut(),
            null_mut(),
        );
        // nextline always segfaults because we can't pass in "optional" parameters that are needed
        let ic = xlib::XCreateIC(im);
        let mut buffer: [c_char; 32] = [0; 32];
        let mut keysm: xlib::KeySym = 0;
        let mut status: xlib::Status = 0;
        let mut i_really_do_not_know: u64 = 32;
        xlib::Xutf8LookupString(
            ic,
            &mut event as *mut xlib::XKeyEvent,
            buffer.as_mut_ptr(),
            //&i_really_do_not_know as *mut u64,
            32,
            &mut keysm as *mut u64,
            &mut status as *mut i32,
        );
        println!("{:?}", buffer);
    }
}

impl XorgUserInterface {
    pub fn new() -> Result<XorgUserInterface, Box<dyn std::error::Error>> {
        let (conn, screen_num) = XCBConnection::connect(None)?;
        let screen = &conn.setup().roots[screen_num];
        let atoms = AtomCollection::new(&conn)?.reply()?;
        let (mut width, mut height) = (screen.width_in_pixels, 50);
        let (depth, visualid) = choose_visual(&conn, screen_num)?;

        // Check if a composite manager is running. In a real application, we should also react to a
        // composite manager starting/stopping at runtime.
        let transparency = composite_manager_running(&conn, screen_num)?;

        let window = create_window(&conn, &screen, &atoms, (width, height), depth, visualid)?;

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

        // grab keyboard
        let root = screen.root;
        let cookie = conn
            .grab_keyboard(true, root, 0 as u32, GrabMode::ASYNC, GrabMode::ASYNC)
            .unwrap();

        if cookie.reply().unwrap().status != GrabStatus::SUCCESS {
            return Err(Box::from(KeyboardGrabError::new("failed grab keyboard")));
        }

        Ok(XorgUserInterface {
            connection: conn,
            window,
            screen: screen_num,
            surface,
            atoms,
            width,
            height,
            transparency,
        })
    }
}

impl UserInterface for XorgUserInterface {
    fn run(&mut self, menu: &mut Menu) -> Result<(), Box<dyn std::error::Error>> {
        let mut stop = false;
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
                            println!("Window was asked to close");
                            return Ok(());
                        }
                    }
                    Event::KeyPress(event) | Event::KeyRelease(event) => {
                        match handle_keyboard(event, menu) {
                            XorgUiAction::Stop => {
                                stop = true;
                            }
                            XorgUiAction::Redraw => {
                                need_redraw = true;
                            }
                            XorgUiAction::None => {}
                        };
                    }
                    Event::Error(_) => println!("Got an unexpected error"),
                    _ => println!("Got an unknown event"),
                }
                event_option = self.connection.poll_for_event()?;
            }
            if need_redraw {
                let cr = cairo::Context::new(&self.surface);
                do_draw(
                    &cr,
                    (self.width as _, self.height as _),
                    self.transparency,
                    &menu,
                );
                self.surface.flush();
            }
            if stop {
                return Ok(());
            }
        }
    }
}
