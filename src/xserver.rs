extern crate libc;
use xlib::XKeysymToString;
use self::libc::funcs::c95::stdlib::malloc;
use self::libc::{c_int,c_uint};
use std::mem::uninitialized;
pub use xlib::*;
pub use key::{
    Key,
    KeyMod,
    Shift,
    Lock,
    Control,
    Mod1,
    Mod2,
    Mod3,
    Mod4,
    Mod5,
};
use config::KeyBind;
use std::c_str::CString;
use std::mem::size_of;
use std::ptr::{null,null_mut};

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
    pub fn XkbKeycodeToKeysym(dpy: *const Display, kc: KeyCode, group: uint, level: uint) -> KeySym;
}

//This is not going to be window server agnostic like wtftw is. Sorry, too hard
//to tell what is server specific or not...Also I don't use Wayland
pub struct XServer {
    pub display: *mut Display, //Why is this called Display, it's a Connection
    root: Window,
    event: *mut XEvent,
    keys: Vec<Key>
}

pub enum ServerEvent {
    ButtonPressed,
    KeyPressed
}

pub fn keysym_to_string(sym: KeySym) -> CString {
    unsafe {
        let s: *const i8 = &*XKeysymToString(sym);
        //I don't know enough about Rust FFI to know if this will memory leak, but I fear it will
        CString::new(s,false)
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
        debug!("XServer display: {}",disp);
        let root = XRootWindow(disp, screen_num);
        XSelectInput(disp,root,SubstructureNotifyMask|SubstructureRedirectMask|PropertyChangeMask|KeyPressMask | KeyReleaseMask | ButtonPressMask);
        XSync(disp,1);
        XServer {
            display: disp,
            root: root,
            event: malloc(256),//unsafe { *malloc(size_of::<XEvent>() as *const XEvent) }, //Stolen from wtftw(ish)
            keys: Vec::new()
        }
    }

    fn get_event_as<T>(&self) -> *mut T { //XEvent as XKeyPressedEvent
        let ev: *mut XEvent = self.event;
        unsafe {
            ev as *mut T
        }
    }

    pub fn get_event(&mut self) -> ServerEvent {
        debug!("Calling XNextEvent");
        debug!("disp: {} event: {}", self.display, self.event);
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
                println!("Pressed button {}",(*ev).state);
                ButtonPressed
            },
            KeyPress => unsafe {
                let ev = self.get_event_as::<XKeyPressedEvent>();
                println!("Key pressed {}",(*ev).keycode);
                KeyPressed
            },
            _ => ButtonPressed
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
                XGrabKey(self.display, key.code as i32, (key.modifier | Lock).bits(), self.root, true as i32, GrabModeAsync, GrabModeAsync);
            }
        }
    }

    pub fn clear_keys(&mut self){
        self.keys.clear();
        unsafe {
            XUngrabKey(self.display, 0 /*AnyKey*/, (1<<15) /*AnyModifier*/, self.root);
        }
    }

    //There isn't a very good place for this.
    //It was in config, but it would be magically affecting the keys Vec<T>
    pub fn parse_key(&mut self, k: &mut KeyBind) -> Key {
        if k.binding.is_none() {
            k.binding = Some(Key::create(k.chord.clone(),self));
            self.add_key(k.binding.unwrap());
            k.binding.unwrap()
        }
        else {
            k.binding.unwrap()
        }
    }

    pub fn string_to_keysym(&self,s: &String) -> KeySym {
        unsafe {
            let mut x = s.to_c_str();
            XStringToKeysym(x.as_mut_ptr())
        }
    }

    fn keycode_to_keysym(&self,code: KeyCode) -> KeySym {
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