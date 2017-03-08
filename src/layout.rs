use xserver::{XServer};
use super::Gallium;
use window_manager::{Workspace,Window};
use deck::Deck;
use config::{Config,Direction,SpecialMsg};
extern crate core;
use self::core::ops::IndexMut;

//Why is this not provided by the stdlib?
pub fn clamp<T: Ord>(min: T, v: T, max:T) -> T {
    if max<=min {
        return min //kinda makes sense, and makes my life easier
    }
    if v<min {
        return min
    }
    if v>max {
        return max
    }
    v
}

//Accumulator-ish region, to keep current position book-keeping for layouts
struct Region {
    x: usize,
    y: usize,
    size_x: usize,
    size_y: usize
}

pub trait Layout {
    fn apply(&self, screen: u32, xserv: &mut XServer, work: &mut Workspace, conf: &mut Config){
        println!("Layout.apply unimplemented");
    }
    fn message(&mut self, SpecialMsg){
        println!("Layout.special unimplemented");
    }
}

#[derive(Clone,Copy,PartialEq,RustcDecodable,RustcEncodable,Debug)]
pub enum Layouts {
    Tall,
    Wide,
    Grid,
    Stacking,
    Full
}
pub fn LayoutFactory(form: Layouts) -> Box<Layout> {
    let lay: Box<Layout> = match form {
        Layouts::Tall => Box::new(TallLayout {
            master: 1,
            percent: 50.0
        }),
        Layouts::Full => Box::new(FullLayout),
        _ => panic!("Unimplemented layout form!")
    };
    lay
}

//Placeholder layout for swapping
pub struct HolderLayout;
impl Layout for HolderLayout { }

/*
 * Like spectrwm's tall layout.
 * Two columns, master and overflow.
 * Can increase the number of windows that are in master,
 * and can also shift the seperator between the columns.
 * Overflow is the remainder, split vertically for each window.
*/
pub struct TallLayout {
    //How many windows are in the master column.
    pub master: u16,
    //Percent of the screen that master takes up
    pub percent: f32
}

impl Layout for TallLayout {
    fn apply(&self, screen: u32, xserv: &mut XServer, work: &mut Workspace, config: &mut Config){
        {
        let mut wind: &mut Vec<&mut Window> = &mut work.windows.cards[..]
            .iter_mut().filter(|wind| !wind.floating ).collect();
        let pad = config.padding as usize;
        let (x,y) = (xserv.width(screen as u32) as usize - pad - pad,
                    xserv.height(screen as u32) as usize - pad - pad);
        let space = config.spacing as usize;
        let mast = clamp(0,self.master as usize,wind.len());
        debug!("Master {}",mast);
        if wind.len() == 0 {
            trace!("Empty window stack");
            return;
        }
        let mut mast_size = clamp(space,
                              ((x as f32)*(self.percent/100.0)).floor() as usize,
                              x) - space;
        let mut leftover = x-(mast_size+space);
        if wind.len()<=mast { //Don't show a gap if we have no overflow
            leftover = 0;
            mast_size = x;
        }
        else if mast == 0 { //Or if we have no master
            mast_size = pad-space; //This is a hack. We can't have a space!
            leftover = x;
        }
        let mut reg = Region { x: pad, y: pad, size_x: mast_size, size_y: y };
        for m in 0..mast {
            let ref mut w = wind.index_mut(m);
            //We are already offset by pad, dont need another space if we are the top
            let kill_s = if m==0 || m==(mast) { 0 } else { space };
            //If the spacing doesn't divide evenly, stretch the top's Y
            let size_extra = if m==0 { (reg.size_y-(mast*space))%mast } else { 0 }; //Terrible
            let wind_y = (reg.size_y/mast) - kill_s + size_extra;
            reg.y = reg.y + kill_s;
            w.x = reg.x as isize;
            w.y = reg.y as isize;
            w.size = (reg.size_x as usize, wind_y as usize);
            w.shown = true;
            reg.y = reg.y + wind_y;
            xserv.refresh(w);
        }

        let mut reg = Region { x: (x-leftover)+space, y: pad, size_x: leftover, size_y: y };
        let wlen = wind.len();
        if wlen-mast as usize > 0 {
            let stack = wlen-mast as usize;
            for r in mast..wlen {
                let ref mut w = wind[r as usize];
                let kill_s = if r==mast || r==(wlen) { 0 } else { space };
                let size_extra = if r==mast { (reg.size_y-(stack*space))%stack } else { 0 }; //Terrible
                let wind_y = (reg.size_y/stack) - kill_s + size_extra;
                reg.y = reg.y + kill_s;
                w.x = reg.x as isize;
                w.y = reg.y as isize;
                w.size = (reg.size_x as usize,wind_y as usize);
                w.shown = true;
                reg.y = reg.y + wind_y;
                xserv.refresh(w);
            }
        }
        }  // non-lexical borrows when
        for wind in work.windows.cards.iter_mut().filter(|wind| wind.floating) {
            wind.shown = true;
            xserv.refresh(wind);
        }
        work.windows.index.map(|wind|{
            let ref mut wind = &mut work.windows.cards[wind];
            wind.shown = true;
            xserv.refresh(wind);
            unsafe { xserv.bring_to_front(wind.wind_ptr); } // XX MAY BE HORRIBLY WRONG
        });
    }

    fn message(&mut self, msg: SpecialMsg){
        match msg {
            SpecialMsg::Add => if self.master < 5 {
                self.master+=1;
            },
            SpecialMsg::Subtract => if self.master > 0 {
                self.master-=1;
            },
            SpecialMsg::Shrink => {
                if self.percent > 5.0 {
                    self.percent = (self.percent - 5.0).floor();
                }
            },
            SpecialMsg::Grow => {
                if self.percent < 95.0 {
                    self.percent = (self.percent + 5.0).floor();
                }
            }
        }
    }
}

pub struct FullLayout;

impl Layout for FullLayout {
    fn apply(&self, screen: u32, xserv: &mut XServer, work: &mut Workspace, conf: &mut Config){
        let pad = conf.padding as usize;
        let (x,y) = (xserv.width(screen as u32) as usize - pad - pad,
                    xserv.height(screen as u32) as usize - pad - pad);
        let space = conf.spacing as usize;
        let focus = work.windows.index;
        let mut count = work.windows.cards.len();
        for (ind,ref mut wind) in work.windows.cards.iter_mut().enumerate() {
            wind.x = (pad + (space * ind)) as isize;
            wind.y = (pad + (space * ind)) as isize;
            wind.size = (clamp(0,x - (space * count),x) as usize,
                         clamp(0,y - (space * count),y) as usize);
            wind.shown = true;
            xserv.refresh(wind);
        }
        focus.map(|wind|{
            let ref mut wind = &mut work.windows.cards[wind];
            wind.shown = true;
            xserv.refresh(wind);
            unsafe { xserv.bring_to_front(wind.wind_ptr); } // XX MAY BE HORRIBLY WRONG
        });
    }
    fn message(&mut self, msg: SpecialMsg){
        println!("Layout.special unimplemented");
    }
}
