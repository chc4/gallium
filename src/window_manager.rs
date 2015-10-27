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
use std::ops::Index;

pub struct Cycle {
        max: isize,
        slice: Vec<Option<usize>>
}

impl Index<isize> for Cycle {
        type Output=Option<usize>;
        fn index(&self, index: isize) -> &Option<usize> {
                println!("Index {}",index);
                if self.max == 0 { //empty interval, return &None
                    return &self.slice[0];
                }
                if index == 0 {
                    return &self.slice[0];
                }
                let b_ind = self.max + ((index.abs()%self.max) * index.abs()/index);
                let b_ind = b_ind%self.max;
                return &self.slice[b_ind as usize];
       }
}
impl Cycle {
    pub fn new(max: usize) -> Cycle {
        let mut v = Vec::new();
        for i in 0..max {
            v.push(Some(i));
        }
        v.push(None);
        Cycle {
            max: max as isize,
            slice: v
        }
    }
}

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
        // If we can avoid nuking index, then just select the next one in line
        if self.index.is_some() && self.index.unwrap() >= ind && self.cards.len() <= self.index.unwrap()  {
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
    master: Option<&'a Window>,
    name: String
}
impl<'a> Workspace<'a>{
    pub fn refresh(&mut self, xserv: &mut XServer, screen: u32, mut config: Config){
        trace!("Refresh");
        //Swap self.layout so no dual-&mut
        let mut holder: Box<Layout> = Box::new(HolderLayout);
        swap(&mut holder, &mut self.layout);
        holder.apply(screen, xserv, self, &mut config);
        //And restore it
        swap(&mut self.layout, &mut holder);
        let mast = self.windows.current().map(|x| x.wind_ptr );
        for w in &self.windows.cards[..] {
            if mast.is_some() && w.wind_ptr == mast.unwrap() {
                xserv.set_border_color(w.wind_ptr,config.focus_color.0 as u64);
                xserv.focus(w.wind_ptr);
            }
            else {
                xserv.set_border_color(w.wind_ptr,config.unfocus_color.0 as u64);
            }
        }
    }
}

pub struct Window {
    pub wind_ptr: XWindow,
    pub floating: bool,
    pub shown: bool, //not always true! just used with xserv.refresh(window)
    pub _mapped: bool, //used for xserv.map/unmap so it doesn't configure twice
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
        for wind_conf in config.current().workspaces {
            let wind_deck = Deck::new();
            let work = Workspace {
                windows: wind_deck,
                layout: LayoutFactory(wind_conf.layout),
                master: None,
                name: wind_conf.name
            };
            works.push(work);
        }
        works.select(0);
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
    pub fn switch(&mut self, xserv: &mut XServer, mut config: Config, ind: u32) {
        // All of these unwraps should be safe, we will never be at an empty workspace
        if ind as usize >= self.workspaces.cards.len() {
            panic!("Invalid workspace {} to switch to",ind);
        }
        //First, unmap all current windows
        for w in self.workspaces.current().unwrap().windows.cards.iter_mut() {
            w.shown = false;
            xserv.refresh(w);
        }
        //Switch workspaces
        self.workspaces.select(ind as usize);
        //And finally let the layout remap all the ones that it wants
        //(The layout shouldn't have to do anything fancy, since
        // xserv.refresh(window) will map the window)
        self.workspaces.current().unwrap().refresh(xserv,self.screens.index.unwrap() as u32,config);
    }
}

/*

WindowManager has a list of screens and a list of workspaces.
The current screen should be a member of this.
Each screen has a workspace index into the list, too.

*/
