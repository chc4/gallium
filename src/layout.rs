use xserver::{XServer};
use super::Gallium;
use window_manager::{Deck,Workspace,Window};
use config::{Config,Direction};

//Why is this not provided by the stdlib?
fn clamp<T: Ord>(min: T, v: T, max:T) -> T {
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

pub trait Layout {
    fn apply(&self, screen: u32, xserv: &mut XServer, work: &mut Workspace, conf: &mut Config){
        println!("Layout.apply unimplemented");
    }
    fn add(&mut self, &mut Window){
        println!("Layout.add unimplemented");
    }
    fn remove(&mut self, usize){
        println!("Layout.remove unimplemented");
    }
    fn special(&mut self, Direction){
        println!("Layout.special unimplemented");
    }
}

#[derive(Copy,PartialEq)]
pub enum Layouts {
    Tall,
    Wide,
    Grid,
    Stacking,
    Full
}
pub fn LayoutFactory(form: Layouts) -> Box<Layout> {
    let lay = match form {
        Layouts::Tall => TallLayout {
            master: 1,
            percent: 50.0
        },
        _ => panic!("Unimplemented layout form!")
    };
    Box::new(lay)
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
        let mut wind = &mut work.windows.cards[..];
        let (x,y) = (xserv.width(screen as u32) as usize,xserv.height(screen as u32) as usize);
        let mast = clamp(0,self.master as usize,wind.len());
        println!("Master: {} {}",mast,self.master);
        if wind.len() == 0 {
            println!("Empty window stack");
            return;
        }
        let mut mast_size = clamp(0,
                              ((x as f32)*(self.percent/100.0)).floor() as usize,
                              x);
        let mut leftover = x-mast_size;
        if wind.len()<=mast { //Don't show a gap if we have no overflow
            leftover = 0;
            mast_size = x;
        }
        else if mast == 0 { //Or if we have no master
            mast_size = 0;
            leftover = x;
        }
        for m in 0..mast {
            println!("Applying layout to window {} (master)",m);
            let ref mut w = wind[m];
            w.x = 0;
            w.y = ((y/mast)*m) as isize;
            w.size = (mast_size as usize,(y/mast) as usize);
            xserv.refresh(w);
        }
        if wind.len()-mast as usize > 0 {
            let stack = wind.len()-mast as usize;
            for r in mast..wind.len() {
                println!("Appling layout to window {}",r);
                let ref mut w = wind[r as usize];
                w.x = mast_size as isize;
                w.y = ((y/stack)*(r-mast)) as isize;
                w.size = (leftover as usize,(y/stack) as usize);
                xserv.refresh(w);
            }
        }
    }

    fn special(&mut self, dir: Direction){
        match dir {
            //These are technically backwards...
            //Just imagine it as "subtracting" from the overflow?
            Direction::Down => if self.master < 5 {
                self.master+=1;
            },
            Direction::Up => if self.master > 0 {
                self.master-=1;
            },
            //FIXME: this should be under Grow/Shrink!
            Direction::Backward => {
                if self.percent > 5.0 {
                    self.percent = (self.percent - 5.0).floor();
                }
            },
            Direction::Forward => {
                if self.percent < 95.0 {
                    self.percent = (self.percent + 5.0).floor();
                }
            }
        }
    }

    fn add(&mut self, wind: &mut Window){
        println!("Added a new window, yaaay");
    }
    fn remove(&mut self, ind: usize){
        println!("Removed window {}, oh nooo",ind);
    }
}
