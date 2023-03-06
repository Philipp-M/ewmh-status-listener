use serde::Serialize;
use xcb::Xid;
use xcb_wm::ewmh;

#[derive(Debug, Clone, Serialize)]
struct Desktop {
    name: String,
    id: u32,
    windows: Vec<Window>,
}

#[derive(Debug, Clone, Copy, Serialize)]
enum WindowState {
    Modal,
    Sticky,
    MaximizedVert,
    MaximizedHorz,
    Shaded,
    SkipTaskbar,
    SkipPager,
    Hidden,
    Fullscreen,
    Above,
    Below,
    DemandsAttention,
}

#[derive(Debug, Clone, Serialize)]
struct Window {
    resource_id: u32,
    name: String,
    desktop_id: u32,
    states: Vec<WindowState>,
}

#[derive(Debug, Clone, Serialize)]
struct State {
    desktops: Vec<Desktop>,
    current_desktop_id: u32,
    active_window: Option<Window>,
}

fn state(conn: &ewmh::Connection) -> xcb::Result<State> {
    use ewmh::proto::*;
    let mut desktops: Vec<_> = conn
        .wait_for_reply(conn.send_request(&ewmh::proto::GetDesktopNames))?
        .names
        .into_iter()
        .enumerate()
        .map(|(id, name)| Desktop { name, id: id as u32, windows: vec![] })
        .collect();

    let active_xcb_window = conn.wait_for_reply(conn.send_request(&GetActiveWindow))?.window;
    let mut active_window = None;

    let client_list = conn.wait_for_reply(conn.send_request(&GetClientList))?;

    for client in client_list.clients {
        let window = window(conn, client)?;
        if active_xcb_window == client {
            active_window = Some(window.clone());
        }
        desktops[window.desktop_id as usize].windows.push(window);
    }

    let current_desktop_id = conn.wait_for_reply(conn.send_request(&GetCurrentDesktop))?.desktop;

    Ok(State { desktops, active_window, current_desktop_id })
}

fn window(conn: &ewmh::Connection, window: xcb::x::Window) -> xcb::Result<Window> {
    use ewmh::proto::*;
    let desktop_id = conn.wait_for_reply(conn.send_request(&GetWmDesktop(window)))?.desktop;
    let cur_win_state = conn.wait_for_reply(conn.send_request(&GetWmState(window)))?;
    let name = conn.wait_for_reply(conn.send_request(&GetWmName(window)))?.name;
    let states = cur_win_state
        .states
        .iter()
        .filter_map(|state| {
            if state == &conn.atoms._NET_WM_STATE_MODAL {
                Some(WindowState::Modal)
            } else if state == &conn.atoms._NET_WM_STATE_STICKY {
                Some(WindowState::Sticky)
            } else if state == &conn.atoms._NET_WM_STATE_MAXIMIZED_VERT {
                Some(WindowState::MaximizedVert)
            } else if state == &conn.atoms._NET_WM_STATE_MAXIMIZED_HORZ {
                Some(WindowState::MaximizedHorz)
            } else if state == &conn.atoms._NET_WM_STATE_SHADED {
                Some(WindowState::Shaded)
            } else if state == &conn.atoms._NET_WM_STATE_SKIP_TASKBAR {
                Some(WindowState::SkipTaskbar)
            } else if state == &conn.atoms._NET_WM_STATE_SKIP_PAGER {
                Some(WindowState::SkipPager)
            } else if state == &conn.atoms._NET_WM_STATE_HIDDEN {
                Some(WindowState::Hidden)
            } else if state == &conn.atoms._NET_WM_STATE_FULLSCREEN {
                Some(WindowState::Fullscreen)
            } else if state == &conn.atoms._NET_WM_STATE_ABOVE {
                Some(WindowState::Above)
            } else if state == &conn.atoms._NET_WM_STATE_BELOW {
                Some(WindowState::Below)
            } else if state == &conn.atoms._NET_WM_STATE_DEMANDS_ATTENTION {
                Some(WindowState::DemandsAttention)
            } else {
                None
            }
        })
        .collect();
    Ok(Window { resource_id: window.resource_id(), name, desktop_id, states })
}

fn print_state(conn: &ewmh::Connection) -> xcb::Result<()> {
    println!("{}", serde_json::to_string(&state(conn)?).unwrap());
    Ok(())
}

fn main() -> xcb::Result<()> {
    // TODO just loop/timeout/wait if an error occurs?
    let (conn, _screen) = xcb::Connection::connect(None)?;
    let ewmh_con = ewmh::Connection::connect(&conn);

    // print first state for initial setup (e.g. in eww)
    print_state(&ewmh_con)?;

    // setup connection to get every property change event (similar as "xprop -root -spy")
    for screen in conn.get_setup().roots() {
        use xcb::x::*;
        let window = screen.root();
        let value_list = &[Cw::EventMask(EventMask::PROPERTY_CHANGE)];
        conn.send_request(&ChangeWindowAttributes { window, value_list });
    }
    conn.flush()?;

    loop {
        let event = conn.wait_for_event()?;
        let event = if let xcb::Event::X(xcb::x::Event::PropertyNotify(event)) = &event {
            event
        } else {
            continue;
        };

        // TODO maybe try to group/buffer multiple intermedite events
        // (e.g. _NET_CLIENT_LIST_STACKING and _NET_ACTIVE_WINDOW are almost always executed together in xmonad at least)
        let atom = event.atom();
        if atom == ewmh_con.atoms._NET_ACTIVE_WINDOW
            || atom == ewmh_con.atoms._NET_CURRENT_DESKTOP
            || atom == ewmh_con.atoms._NET_CLIENT_LIST_STACKING
            || atom == ewmh_con.atoms._NET_CLOSE_WINDOW
            || atom == ewmh_con.atoms._NET_SHOWING_DESKTOP
        {
            print_state(&ewmh_con)?;
        }
    }
}
