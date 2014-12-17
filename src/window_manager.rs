use xserver::{XServer,Window,Display,Screen,XDefaultScreenOfDisplay};
use layout::{Layouts};
use config::{Config,ConfigLock};
use std::rc::Rc;
use std::cell::RefCell;
use std::slice::PartialEqSlicePrelude;
use std::cmp::PartialEq;
use std::ptr;
use std::mem;

//A Vec<T> that has a `current` member.
//Selecting a new current will move that card to the top of the deck
pub struct Deck<T>{
    cards: Vec<Rc<RefCell<T>>>, //If you get weird mutability errors, RefCell<T>
    current: Option<Rc<RefCell<T>>> 
}

impl<T: Eq> Deck<T>{
    fn new() -> Deck<T>{
        Deck {
            cards: Vec::new(),
            current: None
        }
    }
    pub fn push(&mut self,card: T){
        let re = Rc::new(RefCell::new(card));
        self.cards.push(re.clone());
        self.current = Some(re);
    }
    pub fn pop(&mut self) -> Option<Rc<RefCell<T>>>{
        let r = self.cards.pop();
        let ref curr = self.current.clone();
        if curr.is_some() && r.is_some() && curr.clone().unwrap() == r.clone().unwrap() {
            self.current = None;
        }
        r
    }
    fn swap(&mut self,pos1: uint,pos2: uint){
        self.cards.swap(pos1,pos2);
    }
    //This is O(n) and re-allocates everything right of the index. It's bad.
    fn remove(&mut self,card: &Rc<RefCell<T>>) -> Option<Rc<RefCell<T>>> {
        let ind = self.cards.as_slice().position_elem(card);
        if ind.is_none() {
            return None
        }
        let k = self.cards.remove(ind.unwrap());
        if k.is_some() && k.clone().unwrap().eq(card) {
            self.current = None
        }
        k
    }
}
//WorkspaceManager is ~indirection~
//WorkspaceManager
//    -> many Displays
//        -> Workspaces mapped to Displays
//            -> Windows mapped to Workspaces
pub struct Workspace<'a> {
    pub windows: Deck<Window>, //Is treated as a stack of the most recent windows
    pub layout: Layouts,
    master: Option<&'a Window>
}
impl<'a> Workspace<'a>{
    pub fn refresh(&mut self){
        //Will re-apply the layout to the windows, but not implemented yet
        println!("Refresh");
    }
}

pub struct Monitor<'a> {
    //Always one display, but mutable workspace + layout bound to it
    screen: Screen,
    pub workspace: uint,
}

//Like jesus christ I hate everything.
//I tried out lifetimes + copy_lifetime and after like 45 minutes of wrestling
//with the borrow checker, it compiled. And then immediately segfaults when I try to push.
pub struct WindowManager<'a> {
    screens: Vec<Monitor<'a>>,
    pub curr_screen: uint,
    pub curr_workspace: uint,
    workspaces: Vec<Workspace<'a>> //Vec<Workspace>? List of all workspaces so we can increment each Screen
}

impl<'a> WindowManager<'a> {
    /// Return the current workspace
    pub fn workspace(&mut self) -> &mut Workspace<'a>{
       let c_w = self.curr_workspace;
       &mut self.workspaces[c_w]
    }
    /// Returns the current screen
    pub fn screen(&mut self) -> &mut Monitor<'a>{
        let c_s = self.curr_screen;
        &mut self.screens[c_s]
    }
    pub fn new(serv: &XServer, config: &ConfigLock) -> WindowManager<'a> {
        //TODO: Figure out how the heck XineramaScreenInfo is used for this
        //Preferably without having to recreate the Screen class to use it
        //let xine_enabled = unsafe { XineramaQueryExtension(serv.display) };
        //let num_screens = 1;
        let mut screens = Vec::new();
        let mut works = Vec::new();
        /*if xine_enabled {
            debug!("Xinerama is enabled!");
            let screens_: *const XineramaScreenInfo = unsafe { XineramaQueryScreens(serv.display, &num_screens) };
            XFree(screens_);
        }*/
        let mut wind_deck = Deck::new();
        let mut work = Workspace {
            windows: wind_deck,
            layout: Layouts::Tall,
            master: None
        };
        works.push(work);
        let mut scr = Monitor {
            screen: unsafe { ptr::read(&*XDefaultScreenOfDisplay(serv.display.clone())) }, //Reused from XServer::new() :(
            workspace: 0
        };
        screens.push(scr);
        WindowManager {
            screens: screens,
            curr_screen: 0,
            curr_workspace: 0,
            workspaces: works
        }
    }
}

/*

WindowManager has a list of screens. The current screen should be a member of this.
wm.current should probably be a refcell into the screens Vec

*/
