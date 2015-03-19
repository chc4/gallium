#[allow(non_upper_case_globals)]
extern crate libc;
use std::cell::RefCell;
use std::old_path::BytesContainer;
use self::libc::funcs::c95::stdlib::malloc;
pub use xlib::*;
use window_manager::Window as GWindow;
pub use xlib::Window as XWindow;
pub use key::{
    Key,
    KeyMod,
    SHIFT,
    LOCK,
    CONTROL,
    MOD1,
    MOD2,
    MOD3,
    MOD4,
    MOD5 
};
use config::KeyBind;
use std::ffi::{CString,CStr,c_str_to_bytes};
use std::ptr::{null_mut,read};
use std::mem::{zeroed,transmute};
use self::libc::{c_long,c_ulong,c_void,c_int};

//Event types
const KeyPress: i32       = 2i32;
const KeyRelease: i32       = 3i32;
const ButtonPress: i32      = 4i32;
const ButtonRelease: i32     = 5i32;
const MotionNotify: i32     = 6i32;
const EnterNotify: i32      = 7i32;
const LeaveNotify: i32      = 8i32;
const FocusIn: i32          = 9i32;
const FocusOut: i32         = 10i32;
const KeymapNotify: i32     = 11i32;
const Expose: i32           = 12i32;
const GraphicsExpose: i32   = 13i32;
const NoExpose: i32         = 14i32;
const VisibilityNotify: i32 = 15i32;
const CreateNotify: i32     = 16i32;
const DestroyNotify: i32    = 17i32;
const UnmapNotify: i32      = 18i32;
const MapNotify: i32        = 19i32;
const MapRequest: i32       = 20i32;
const ReparentNotify: i32   = 21i32;
const ConfigureNotify: i32  = 22i32;
const ConfigureRequest: i32 = 23i32;
const GravityNotify: i32    = 24i32;
const ResizeRequest: i32    = 25i32;
const CirculateNotify: i32  = 26i32;
const CirculateRequest: i32 = 27i32;
const PropertyNotify: i32   = 28i32;
const SelectionClear: i32   = 29i32;
const SelectionRequest: i32 = 30i32;
const SelectionNotify: i32  = 31i32;
const ColormapNotify: i32   = 32i32;
const ClientMessage: i32    = 33i32;
const MappingNotify: i32    = 34i32;
const GenericEvent: i32     = 35i32;

//Input masks
const NoEventMask: i64         = 0i64;
const KeyPressMask: i64            = (1i64<<0);
const KeyReleaseMask: i64          = (1i64<<1);
const ButtonPressMask: i64         = (1i64<<2);
const ButtonReleaseMask: i64       = (1i64<<3);
const EnterWindowMask: i64         = (1i64<<4);
const LeaveWindowMask: i64         = (1i64<<5);
const PointerMotionMask: i64       = (1i64<<6);
const PointerMotionHintMask: i64       = (1i64<<7);
const Button1MotionMask: i64       = (1i64<<8);
const Button2MotionMask: i64       = (1i64<<9);
const Button3MotionMask: i64       = (1i64<<10);
const Button4MotionMask: i64       = (1i64<<11);
const Button5MotionMask: i64       = (1i64<<12);
const ButtonMotionMask: i64        = (1i64<<13);
const KeymapStateMask: i64         = (1i64<<14);
const ExposureMask: i64            = (1i64<<15);
const VisibilityChangeMask: i64        = (1i64<<16);
const StructureNotifyMask: i64     = (1i64<<17);
const ResizeRedirectMask: i64      = (1i64<<18);
const SubstructureNotifyMask: i64      = (1i64<<19);
const SubstructureRedirectMask: i64    = (1i64<<20);
const FocusChangeMask: i64         = (1i64<<21);
const PropertyChangeMask: i64      = (1i64<<22);
const ColormapChangeMask: i64      = (1i64<<23);
const OwnerGrabButtonMask: i64     = (1i64<<24);

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
    Unknown
}

pub fn keysym_to_string(sym: KeySym) -> &'static CStr {
    unsafe {
        let s: *const i8 = XKeysymToString(sym);
        //I don't know enough about Rust FFI to know if this will memory leak, but I fear it will
        //CString::from_slice(&*(s as *const [u8]))
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
        //XSelectInput(disp,root,SubstructureNotifyMask|SubstructureRedirectMask|PropertyChangeMask|KeyPressMask | KeyReleaseMask | ButtonPressMask);
        XSelectInput(disp,root,SubstructureNotifyMask|SubstructureRedirectMask);
        XSync(disp,1);
        XServer {
            display: disp,
            root: root,
            event: malloc(256) as *mut XEvent,//unsafe { *malloc(size_of::<XEvent>() as *const XEvent) }, //Stolen from wtftw(ish)
            keys: Vec::new(),
            kommand_mod: RefCell::new(None)
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
        }
    }

    pub fn refresh(&mut self,gwind: &GWindow){
        unsafe {
            let (s_x,s_y) = gwind.size;
            XMoveResizeWindow(self.display, gwind.wind_ptr, gwind.x as i32, gwind.y as i32, s_x as u32, s_y as u32);
        }
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
            data: data, //There is no way this works
        };
        let mess_ptr = &mut message as *mut XClientBullshit as *mut XClientMessageEvent;
        XSendEvent(self.display, wind, 0 /*false*/, NoEventMask, (mess_ptr as *mut XEvent));
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
        for ind in range(0,nchildren as isize){
            let wind: Window = *children.offset(ind);
            self.kill_window(wind); //Kill them softly
        }
        'stall: while(nchildren > 0){
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
        debug!("Calling XNextEvent");
        debug!("disp: {:?} event: {:?}", self.display, self.event);
        unsafe {
            XNextEvent(self.display, self.event);
        }
        debug!("XNextEvent passed");
        //XEvent is just a c_void right now, since Rust doesn't have C-style unions.
        //We have to get event.type manually. Sorry.
        let event_type = unsafe {
            *(self.event as *mut i32)
        };
        match event_type {
            ButtonPress => unsafe {
                let ev = self.get_event_as::<XButtonPressedEvent>();
                println!("Pressed button {}",ev.state);
                ServerEvent::ButtonPress((ev.x,ev.y))
            },
            KeyPress => unsafe {
                let ev = self.get_event_as::<XKeyPressedEvent>();
                //let m = KeyMod::from_bits(ev.state).unwrap_or(KeyMod::empty());
                //ServerEvent::KeyPress(ev.keycode,m)
                let k = Key::parse(ev.keycode as KeyCode,ev.state,self);
                ServerEvent::KeyPress(k)
            },
            MapRequest => unsafe {
                let ev = self.get_event_as::<XMapRequestEvent>();
                println!("Window added {}",ev.window);
                let w = GWindow {
                   wind_ptr: ev.window,
                   x: 0,
                   y: 0,
                   z: 0,
                   size: (0,0)
                };
                ServerEvent::MapRequest(w)
            },
            DestroyNotify => unsafe {
                let ev = self.get_event_as::<XDestroyWindowEvent>();
                println!("Window destroyed {}",ev.window);
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
            let mut x = CString::from_slice(s.as_mut_vec().as_mut_slice());
            XStringToKeysym(x.as_ptr() as *mut i8)
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
