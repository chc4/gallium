use xserver::{XServer,Window,Display,Screen,XDefaultScreenOfDisplay};
use layout::{Layouts,Tall};
use config::{Config,ConfigLock};
use std::rc::Rc;
use std::cell::RefCell;
use std::slice::PartialEqSlicePrelude;
use std::cmp::PartialEq;

//A Vec<T> that has a `current` member.
//Selecting a new current will move that card to the top of the deck
struct Deck<T>{
    cards: Vec<Rc<T>>, //If you get weird mutability errors, RefCell<T>
    current: RefCell<Option<Rc<T>>>
}

impl<T: Eq> Deck<T>{
    fn new() -> Deck<T>{
        Deck {
            cards: Vec::new(),
            current: RefCell::new(None)
        }
    }
    fn push(&mut self,card: T){
        let re = Rc::new(card);
        self.cards.push(re.clone());
        *self.current.borrow_mut() = Some(re);
    }
    fn pop(&mut self) -> Option<Rc<T>>{
        let r = self.cards.pop();
        let curr = self.current.borrow();
        if curr.is_some() && r.is_some() && curr.clone().unwrap() == r.clone().unwrap() {
            *self.current.borrow_mut() = None;
        }
        r
    }
    //This is O(n) and re-allocates everything right of the index. It's bad.
    fn remove(&mut self,card: &Rc<T>) -> Option<Rc<T>> {
        let ind = self.cards.as_slice().position_elem(card);
        if ind.is_none() {
            return None
        }
        let k = self.cards.remove(ind.unwrap());
        if k.is_some() && k.clone().unwrap().eq(card) {
            *self.current.borrow_mut() = None
        }
        k
    }
}
//WorkspaceManager is ~indirection~
//WorkspaceManager
//    -> many Displays
//        -> Workspaces mapped to Displays
//            -> Windows mapped to Workspaces
struct Workspace {
    windows: Deck<Window>, //Is treated as a stack of the most recent windows
    layout: Layouts,
    master: Option<Rc<Window>>
}

struct Monitor {
    //Always one display, but mutable workspace + layout bound to it
    screen: Screen,
    workspace: Rc<Workspace>,
}
pub struct WindowManager {
    screens: Vec<Rc<Monitor>>,
    selected: Rc<Monitor>,
    workspaces: Vec<Rc<Workspace>> //Vec<Workspace>? List of all workspaces so we can increment each Screen
}

impl WindowManager {
    pub fn new(serv: &XServer, config: &ConfigLock) -> WindowManager {
        //TODO: Figure out how the heck XineramaScreenInfo is used for this
        //Preferably without having to recreate the Screen class to use it
        //let xine_enabled = unsafe { XineramaQueryExtension(serv.display) };
        //let num_screens = 1;
        let mut screens = Vec::new();
        /*if xine_enabled {
            debug!("Xinerama is enabled!");
            let screens_: *const XineramaScreenInfo = unsafe { XineramaQueryScreens(serv.display, &num_screens) };
            XFree(screens_);
        }*/
        let wind_deck = Deck::new();
        let work = Rc::new(Workspace {
            windows: wind_deck,
            layout: Tall,
            master: None
        });
        let scr = Rc::new(Monitor {
            screen: unsafe { *XDefaultScreenOfDisplay(serv.display) }, //Reused from XServer::new() :(
            workspace: work.clone()
        });
        screens.push(scr.clone());
        WindowManager {
            screens: screens,
            selected: scr,
            workspaces: vec!(work.clone())
        }
    }
}

/*

WindowManager has a list of screens. The current screen should be a member of this.
wm.current should probably be a refcell into the screens Vec

*/