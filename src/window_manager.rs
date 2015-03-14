use super::Gallium;
use xserver::{XServer,XWindow,Display,Screen,XDefaultScreenOfDisplay};
use layout::{Layout,Layouts,TallLayout};
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
    pub cards: Vec<T>, //If you get weird mutability errors, RefCell<T>
    pub current: Option<usize> 
}

impl<T> Deck<T>{
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
    pub fn swap(&mut self,pos1: u32,pos2: u32){
        self.cards.swap(pos1 as usize,pos2 as usize);
    }
    //This is O(n) and re-allocates everything right of the index. It's bad.
    pub fn remove(&mut self,ind: u32) -> Option<T> {
        let k = self.cards.remove(ind as usize);
        if self.current.is_some() && self.current.unwrap() >= ind as usize {
            self.current = None
        }
        Some(k)
    }

    fn slice(&mut self) -> &[T] {
        self.cards.as_slice()
    }
}
//WorkspaceManager is ~indirection~
//WorkspaceManager
//    -> many Displays
//        -> Workspaces mapped to Displays
//            -> Windows mapped to Workspaces
pub struct Workspace<'a> {
    pub windows: Deck<Window>, //Is treated as a stack of the most recent windows
    pub layout: Box<Layout>,
    master: Option<&'a Window>
}
impl<'a> Workspace<'a>{
    pub fn refresh(&mut self, xserv: &mut XServer, screen: u32, mut config: Config){
        //Will re-apply the layout to the windows, but not implemented yet
        println!("Refresh");
        self.layout.apply(xserv, screen, &mut self.windows, &mut config);
    }
}

pub struct Window {
    pub wind_ptr: XWindow,
    pub x: isize,
    pub y: isize,
    pub z: isize,
    pub size: (usize,usize)
}

pub struct Monitor {
    //Always one display, but mutable workspace + layout bound to it
    screen: Screen,
    pub workspace: u32,
}

pub struct WindowManager<'a> {
    pub screens: Deck<Monitor>,
    pub workspaces: Deck<Workspace<'a>> //Vec<Workspace>? List of all workspaces so we can increment each Screen
}

impl<'a> WindowManager<'a> {
    pub fn new(serv: &XServer, config: &ConfigLock) -> WindowManager<'a> {
        //TODO: Figure out how the heck XineramaScreenInfo is used for this
        //Preferably without having to recreate the Screen class to use it
        //let xine_enabled = unsafe { XineramaQueryExtension(serv.display) };
        //let num_screens = 1;
        let mut screens = Deck::new();
        let mut works = Deck::new();
        /*if xine_enabled {
            debug!("Xinerama is enabled!");
            let screens_: *const XineramaScreenInfo = unsafe { XineramaQueryScreens(serv.display, &num_screens) };
            XFree(screens_);
        }*/
        let wind_deck = Deck::new();
        let lay = TallLayout {
            columns: 2,
            master: Vec::new()
        };
        let work = Workspace {
            windows: wind_deck,
            layout: Box::new(lay),
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
            workspaces: works
        }
    }
}

/*

WindowManager has a list of screens. The current screen should be a member of this.
wm.current should probably be a refcell into the screens Vec

*/
