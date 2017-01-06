#[allow(non_upper_case_globals)]
extern crate libc;
use std::cell::RefCell;
use libc::malloc;
pub use xlib::*;
use window_manager::Window as GWindow;
pub use xlib::Window as XWindow;
pub use key::{
    Key,
    KeyMod,
    LOCK,
};
use config::KeyBind;
use std::ffi::{CString,CStr};
use std::ptr::{null_mut,read};
use std::mem::{zeroed,transmute};
use self::libc::{c_long,c_ulong,c_void,c_int};

//Event types
mod xevent {
    #![allow(non_upper_case_globals,dead_code)]
    pub const KeyPress: i32       = 2i32;
    pub const KeyRelease: i32       = 3i32;
    pub const ButtonPress: i32      = 4i32;
    pub const ButtonRelease: i32     = 5i32;
    pub const MotionNotify: i32     = 6i32;
    pub const EnterNotify: i32      = 7i32;
    pub const LeaveNotify: i32      = 8i32;
    pub const FocusIn: i32          = 9i32;
    pub const FocusOut: i32         = 10i32;
    pub const KeymapNotify: i32     = 11i32;
    pub const Expose: i32           = 12i32;
    pub const GraphicsExpose: i32   = 13i32;
    pub const NoExpose: i32         = 14i32;
    pub const VisibilityNotify: i32 = 15i32;
    pub const CreateNotify: i32     = 16i32;
    pub const DestroyNotify: i32    = 17i32;
    pub const UnmapNotify: i32      = 18i32;
    pub const MapNotify: i32        = 19i32;
    pub const MapRequest: i32       = 20i32;
    pub const ReparentNotify: i32   = 21i32;
    pub const ConfigureNotify: i32  = 22i32;
    pub const ConfigureRequest: i32 = 23i32;
    pub const GravityNotify: i32    = 24i32;
    pub const ResizeRequest: i32    = 25i32;
    pub const CirculateNotify: i32  = 26i32;
    pub const CirculateRequest: i32 = 27i32;
    pub const PropertyNotify: i32   = 28i32;
    pub const SelectionClear: i32   = 29i32;
    pub const SelectionRequest: i32 = 30i32;
    pub const SelectionNotify: i32  = 31i32;
    pub const ColormapNotify: i32   = 32i32;
    pub const ClientMessage: i32    = 33i32;
    pub const MappingNotify: i32    = 34i32;
    pub const GenericEvent: i32     = 35i32;
}

extern {
    pub fn XkbKeycodeToKeysym(dpy: *const Display, kc: KeyCode, group: usize, level: usize) -> KeySym;
}

//This is not going to be window server agnostic like wtftw is. Sorry, too hard
//to tell what is server specific or not...Also I don't use Wayland
pub struct XServer {
    pub display: *mut Display, //Why is this called Display, it's a Connection
    root: Window,
    event: *mut XEvent,
    keys: Vec<Key>,
    pub kommand_mod: RefCell<Option<KeyMod>>
}

//Actually just a wrapper around XEvents of the same name, but more info
pub enum ServerEvent {
    ButtonPress((i32,i32)),
    KeyPress(Key),
    MapRequest(GWindow),
    DestroyNotify(Window),
    EnterNotify(Window),
    Unknown
}

pub fn keysym_to_string(sym: KeySym) -> &'static CStr {
    unsafe {
        let s: *const i8 = XKeysymToString(sym);
        &CStr::from_ptr(s)
    }
}

impl XServer {
    pub unsafe fn new() -> XServer {
        //TODO: Setup error handler
        let disp = XOpenDisplay(null_mut());
        if disp == null_mut() {
            panic!("No display found, exiting.");
        }
        let screen = XDefaultScreenOfDisplay(disp);
        let screen_num = XDefaultScreen(disp);
        if screen == null_mut() {
            panic!("No default screen found, exiting.");
        }
        debug!("XServer display: {:?}",disp);
        let root = XRootWindow(disp, screen_num);
        XSelectInput(disp,root,(SubstructureNotifyMask|SubstructureRedirectMask|EnterWindowMask).bits());
        XSync(disp,1);
        XServer {
            display: disp,
            root: root,
            event: malloc(256) as *mut XEvent,//unsafe { *malloc(size_of::<XEvent>() as *const XEvent) }, //Stolen from wtftw(ish)
            keys: Vec::new(),
            kommand_mod: RefCell::new(None)
        }
    }

    pub fn focus(&mut self, wind: Window) {
        unsafe {
            XSetInputFocus(self.display,wind,2 /*RevertToParent*/,0 /*CurrentTime*/);
        }
    }

    pub fn set_border_width(&mut self, wind: Window, width: u32) {
       unsafe {
           XSetWindowBorderWidth(self.display,wind,width);
        }
    }

    pub fn set_border_color(&mut self, wind: Window, color: c_ulong) {
        unsafe {
            XSetWindowBorder(self.display,wind,color);
        }
    }

    pub fn width(&mut self, screen: u32) -> u32 {
        unsafe { XDisplayWidth(self.display,screen as i32) as u32 } //why is this i32??
    }
    pub fn height(&mut self, screen: u32) -> u32 {
        unsafe { XDisplayHeight(self.display,screen as i32) as u32 } //why is this i32??
    }

    pub fn map(&mut self,wind: Window){
        unsafe {
            XMapWindow(self.display,wind);
            XSelectInput(self.display,wind,(EnterWindowMask).bits());
        }
    }
    pub fn unmap(&mut self,wind: Window){
        unsafe {
            XUnmapWindow(self.display,wind);
        }
    }

    pub fn refresh(&mut self,gwind: &mut GWindow){
        unsafe {
            if gwind.shown && gwind._mapped == false {
                gwind._mapped = true;
                self.map(gwind.wind_ptr);
            }
            else if gwind.shown == false && gwind._mapped {
                gwind._mapped = false;
                self.unmap(gwind.wind_ptr);
            }

            if gwind._mapped {
                let (s_x,s_y) = gwind.size;
                XMoveResizeWindow(self.display, gwind.wind_ptr, gwind.x as i32, gwind.y as i32, s_x as u32, s_y as u32);
           }
        }
    }

    pub unsafe fn bring_to_front(&mut self, wind: Window){
        //let's hope this is the correct way to do it?
        XConfigureWindow(self.display, wind, /* CWStackMode */ (1<<6),
            &mut XWindowChanges {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
                border_width: 0,
                sibling: 0,
                stack_mode: 0 /* Above */
        });
    }

    pub unsafe fn kill_window(&mut self, wind: Window){
        //Holy shit this is terrible
        let mut data: [c_long; 5] = zeroed();
        let mut wmproto = self.get_atom("WM_PROTOCOLS");
        let mut wmkill = self.get_atom("WM_DELETE_WINDOW");
        data[0] = wmkill as c_long;
        data[1] = 0 as c_long; //CurrentTime
        struct XClientBullshit {
            _type: c_int,
            serial: c_ulong,
            send_event: c_int,
            display: *mut Display,
            window: Window,
            message_type: Atom,
            format: c_int,
            data: [c_long; 5],
        }

        let mut message = XClientBullshit {
            _type: ClientMessage,
            serial: 0,
            send_event: 0,
            display: self.display,
            window: wind,
            message_type: wmproto,
            format: 32,
            data: data,
        };
        let mess_ptr = &mut message as *mut XClientBullshit as *mut XClientMessageEvent;
        XSendEvent(self.display, wind, 0 /*false*/, NoEventMask.bits(), (mess_ptr as *mut XEvent));
    }

    pub unsafe fn quit(&mut self,qkey: Key){
        let mut root_return: Window = zeroed();
        let mut parent: Window = zeroed();
        let mut children: *mut Window = zeroed();
        let mut nchildren: u32 = 0;

        XQueryTree(self.display, self.root, &mut root_return, &mut parent, &mut children, &mut nchildren);
        if nchildren == 0 {
            return;
        }
        for ind in 0..nchildren as isize {
            let wind: Window = *children.offset(ind);
            self.kill_window(wind); //Kill them softly
        }
        'stall: while nchildren > 0 {
            let ev = self.get_event();
            match ev {
                ServerEvent::KeyPress(key) => {
                    if key == qkey { //They pressed the Quit button while waiting
                        //I shall not ask nicely a second time
                        break 'stall;
                    }
                },
                _ => ()
            }
            XQueryTree(self.display, self.root, &mut root_return, &mut parent, &mut children, &mut nchildren);
        }
        self.clear_keys();
    }

    pub fn get_atom(&mut self, s: &str) -> Atom {
        unsafe {
            XInternAtom(self.display, CString::new(s).unwrap().as_ptr() as *mut i8, 1)
        }
    }

    fn get_event_as<T>(&self) -> &T { //XEvent as XKeyPressedEvent
        let ev: *mut XEvent = self.event;
        unsafe {
            &*(ev as *mut T)
        }
    }

    pub fn get_event(&mut self) -> ServerEvent {
        trace!("Calling XNextEvent");
        trace!("disp: {:?} event: {:?}", self.display, self.event);
        unsafe {
            XNextEvent(self.display, self.event);
        }
        trace!("XNextEvent passed");
        let event_type = unsafe { (*self.event)._type };
        match event_type {
            xevent::ButtonPress => unsafe {
                let ev = self.get_event_as::<XButtonPressedEvent>();
                debug!("Pressed button {}",ev.state);
                ServerEvent::ButtonPress((ev.x,ev.y))
            },
            xevent::KeyPress => unsafe {
                let ev = self.get_event_as::<XKeyPressedEvent>();
                //let m = KeyMod::from_bits(ev.state).unwrap_or(KeyMod::empty());
                //ServerEvent::KeyPress(ev.keycode,m)
                let k = Key::parse(ev.keycode as KeyCode,ev.state,self);
                ServerEvent::KeyPress(k)
            },
            xevent::EnterNotify => unsafe {
                let ev = self.get_event_as::<XEnterWindowEvent>();
                ServerEvent::EnterNotify(ev.window)
            },
            xevent::MapRequest => unsafe {
                let ev = self.get_event_as::<XMapRequestEvent>();
                debug!("Window added {}",ev.window);
                let w = GWindow {
                   wind_ptr: ev.window,
                   _mapped: false,
                   shown: false,
                   floating: false,
                   x: 0,
                   y: 0,
                   z: 0,
                   size: (0,0)
                };
                ServerEvent::MapRequest(w)
            },
            xevent::DestroyNotify => unsafe {
                let ev = self.get_event_as::<XDestroyWindowEvent>();
                debug!("Window destroyed {}",ev.window);
                ServerEvent::DestroyNotify(ev.window)
            },
            _ => ServerEvent::Unknown
        }
    }

    pub fn add_key(&mut self, k: Key){
        self.keys.push(k)
    }

    pub fn grab_keys(&mut self){
         unsafe {
            XUngrabKey(self.display, 0 /*AnyKey*/, (1<<15) /*AnyModifier*/, self.root);
        }
        //Apparently this may have problems with NumLock.
        //Cross that bridge when we get to it.
        let GrabModeAsync = 1i32;
        for key in self.keys.iter(){
            unsafe {
                XGrabKey(self.display, key.code as i32, key.modifier.bits(), self.root, true as i32, GrabModeAsync, GrabModeAsync);
                XGrabKey(self.display, key.code as i32, (key.modifier | LOCK).bits(), self.root, true as i32, GrabModeAsync, GrabModeAsync);
            }
        }
    }

    pub fn clear_keys(&mut self){
        self.keys.clear();
        unsafe {
            XUngrabKey(self.display, 0 /*AnyKey*/, (1<<15) /*AnyModifier*/, self.root);
        }
    }

    pub fn string_to_keysym(&self,s: &mut String) -> KeySym {
        unsafe {
            let mut x = CString::new((*s.as_mut_vec()).clone()).unwrap();
            let sym = XStringToKeysym(x.as_ptr() as *mut i8);
            if sym == 0 { //Invalid string!
                println!("{} is an invalid X11 keysym!",s);
                //panic!(format!("{} is an invalid X11 keysym!",s));
            }
            sym
        }
    }

    pub fn keycode_to_keysym(&self,code: KeyCode) -> KeySym {
        unsafe {
            XkbKeycodeToKeysym(&*self.display, code, 0, 0)
        }
    }

    pub fn keysym_to_keycode(&self,sym: KeySym) -> KeyCode {
        unsafe {
            XKeysymToKeycode(self.display, sym) as KeyCode
        }
    }
}
