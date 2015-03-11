use xserver::{XServer,Window,Display,Screen,XDefaultScreenOfDisplay};
use layout::{Layouts};
use config::{Config,ConfigLock};
use std::rc::Rc;
use std::cell::RefCell;
//use std::slice::PartialEqSlicePrelude;
use std::cmp::PartialEq;
use std::ptr;

//A Vec<T> that has a `current` member.
//Selecting a new current will move that card to the top of the deck
#[derive(PartialEq)]
pub struct Deck<T>{
    cards: Vec<T>, //If you get weird mutability errors, RefCell<T>
    current: Option<usize> 
}

impl<T: PartialEq> Deck<T>{
    fn new() -> Deck<T>{
        Deck {
            cards: Vec::new(),
            current: None
        }
    }
    
    pub fn select(&mut self, ind: usize){
        if ind >= self.cards.len() {
            panic!("Selected a card index that doesn't exist");
        }
        self.current = Some(ind)
    }
    pub fn current(&mut self) -> Option<&mut T>{
        if self.current.is_none() || self.current.unwrap() >= self.cards.len() {
            return None
        }
        Some(&mut self.cards[self.current.unwrap()])
    }

    pub fn push(&mut self,card: T){
        self.cards.push(card);
        if self.current.is_none() {
            self.current = Some(self.cards.len()-1);
        }
    }
    pub fn pop(&mut self) -> Option<T>{
        let r = self.cards.pop();
        if self.current.is_some() && self.cards.len() >= self.current.unwrap() {
            self.current = None;
        }
        Some(r.unwrap())
    }
    fn swap(&mut self,pos1: u32,pos2: u32){
        self.cards.swap(pos1 as usize,pos2 as usize);
    }
    //This is O(n) and re-allocates everything right of the index. It's bad.
    fn remove(&mut self,card: &T) -> Option<T> {
        let ind = self.cards.as_slice().position_elem(card);
        if ind.is_none() {
            return None
        }
        let k = self.cards.remove(ind.unwrap());
        if self.current.unwrap() >= ind.unwrap() {
            self.current = None
        }
        Some(k)
    }
}
//WorkspaceManager is ~indirection~
//WorkspaceManager
//    -> many Displays
//        -> Workspaces mapped to Displays
//            -> Windows mapped to Workspaces
#[derive(PartialEq)]
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

pub struct Monitor {
    //Always one display, but mutable workspace + layout bound to it
    screen: Screen,
    pub workspace: u32,
}

//This is now magically index-based because jesus christ
//I blame the person on IRC who told me to use copy_lifetime.
//Because you can't. That's Completely Illegal(tm).
pub struct WindowManager<'a> {
    screens: Vec<Monitor>,
    curr_screen: u32,
    pub workspaces: Deck<Workspace<'a>> //Vec<Workspace>? List of all workspaces so we can increment each Screen
}

impl<'a> WindowManager<'a> {
    pub fn new(serv: &XServer, config: &ConfigLock) -> WindowManager<'a> {
        //TODO: Figure out how the heck XineramaScreenInfo is used for this
        //Preferably without having to recreate the Screen class to use it
        //let xine_enabled = unsafe { XineramaQueryExtension(serv.display) };
        //let num_screens = 1;
        let mut screens = Vec::new();
        let mut works = Deck::new();
        /*if xine_enabled {
            debug!("Xinerama is enabled!");
            let screens_: *const XineramaScreenInfo = unsafe { XineramaQueryScreens(serv.display, &num_screens) };
            XFree(screens_);
        }*/
        let wind_deck = Deck::new();
        let work = Workspace {
            windows: wind_deck,
            layout: Layouts::Tall,
            master: None
        };
        works.push(work);
        let scr = Monitor {
            screen: unsafe { ptr::read(&*XDefaultScreenOfDisplay(serv.display.clone())) }, //Reused from XServer::new() :(
            workspace: 0
        };
        screens.push(scr);
        WindowManager {
            screens: screens,
            curr_screen: 0,
            workspaces: works
        }
    }
}

/*

WindowManager has a list of screens. The current screen should be a member of this.
wm.current should probably be a refcell into the screens Vec

*/
