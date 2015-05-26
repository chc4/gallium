use super::Gallium;
use xserver::{XServer,XWindow,Display,Screen,XDefaultScreenOfDisplay};
use layout::{Layout,Layouts,TallLayout,HolderLayout,LayoutFactory};
use config::{Config,ConfigLock};
use std::rc::Rc;
use std::cell::RefCell;
//use std::slice::PartialEqSlicePrelude;
use std::cmp::PartialEq;
use std::mem::swap;
use std::ptr;

//A Vec<T> that has a `current` member.
#[derive(PartialEq)]
pub struct Deck<T>{
    pub cards: Vec<T>, //If you get weird mutability errors, RefCell<T>
    pub index: Option<usize>
}

impl<T> Deck<T>{
    fn new() -> Deck<T>{
        Deck {
            cards: Vec::new(),
            index: None
        }
    }

    pub fn select(&mut self, ind: usize){
        if ind >= self.cards.len() {
            panic!("Selected a card index that doesn't exist");
        }
        self.index = Some(ind)
    }
    pub fn current(&mut self) -> Option<&mut T>{
        if self.index.is_none() || self.index.unwrap() >= self.cards.len() {
            return None
        }
        Some(&mut self.cards[self.index.unwrap()])
    }

    pub fn push(&mut self,card: T){
        self.cards.push(card);
        if self.index.is_none() {
            self.index = Some(self.cards.len()-1);
        }
    }
    pub fn pop(&mut self) -> Option<T>{
        let r = self.cards.pop();
        if self.index.is_some() && self.cards.len() >= self.index.unwrap() {
            self.index = None;
        }
        Some(r.unwrap())
    }
    pub fn swap(&mut self,pos1: usize,pos2: usize){
        self.cards.swap(pos1,pos2);
    }
    //This is O(n) and re-allocates everything right of the index. It's bad.
    pub fn remove(&mut self,ind: usize) -> Option<T> {
        let k = self.cards.remove(ind);
        if self.index.is_some() && self.index.unwrap() >= ind {
            self.index = None
        }
        Some(k)
    }
    //Remove the element at index, but don't move self.index from it
    //If the index is None or would be OoB, return false
    pub fn forget(&mut self,ind: usize) -> bool {
        let old = self.index.clone();
        let elem = self.remove(ind);
        if old.is_some() && self.index.is_none() {
            if old.unwrap()<self.cards.len() {
                self.select(old.unwrap());
                return true
            }
        }
        false
    }

    fn slice(&mut self) -> &[T] {
        &self.cards[..]
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
        println!("Refresh");
        //Swap self.layout so no dual-&mut
        let mut holder: Box<Layout> = Box::new(HolderLayout);
        swap(&mut holder, &mut self.layout);
        holder.apply(screen, xserv, self, &mut config);
        //And restore it
        swap(&mut self.layout, &mut holder);
        let mast = self.windows.current().map(|x| x.wind_ptr );
        for w in &self.windows.cards[..] {
            if mast.is_some() && w.wind_ptr == mast.unwrap() {
                xserv.set_border_color(w.wind_ptr,config.focus_color as u64);
                xserv.focus(w.wind_ptr);
            }
            else {
                xserv.set_border_color(w.wind_ptr,config.unfocus_color as u64);
            }
        }
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
    pub workspaces: Deck<Workspace<'a>>
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
        let work = Workspace {
            windows: wind_deck,
            layout: LayoutFactory(Layouts::Tall),
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

WindowManager has a list of screens and a list of workspaces.
The current screen should be a member of this.
Each screen has a workspace index into the list, too.

*/
