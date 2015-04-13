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

//Placeholder layout for swapping
pub struct HolderLayout;
impl Layout for HolderLayout {
    fn apply(&self, screen: u32, xserv: &mut XServer, work: &mut Workspace, config: &mut Config){
        panic!("Called Layout.apply on placeholder!");
    }
    fn add(&mut self, wind: &mut Window){
        panic!("Called Layout.add on placeholder!");
    }
    fn remove(&mut self, s: usize){
        panic!("Called Layout.remove on placeholder!");
    }
    fn special(&mut self, dir: Direction){
        panic!("Called Layout.special on placeholder!");
    }
}

pub struct TallLayout {
    //How many splits there are.
    pub columns: u16,
    pub master: Vec<u32>
}

impl Layout for TallLayout {
    fn apply(&self, screen: u32, xserv: &mut XServer, work: &mut Workspace, config: &mut Config){
        let mut wind = &mut work.windows.cards[..];
        let (x,y) = (xserv.width(screen as u32) as usize,xserv.height(screen as u32) as usize);

        let mut col = clamp(1,self.columns as usize,wind.len());
        let mcol = (col-1); //I use this a lot
        println!("Col: {}",col);
        for c in 0..mcol {
            println!("Applying layout to window {} (master)",c);
            let ref mut w = wind[c];
            w.x = ((x/col)*c) as isize;
            w.y = 0;
            w.size = ((x/col) as usize,y);
            xserv.refresh(w);
        }
        if wind.len()-mcol as usize > 0 {
            let stack = wind.len()-mcol as usize;
            for r in mcol..wind.len() {
                println!("Appling layout to window {}",r);
                let ref mut w = wind[r as usize];
                w.x = ((x/col)*mcol) as isize;
                w.y = ((y/(stack))*(r-mcol)) as isize;
                w.size = ((x/col) as usize,(y/stack) as usize);
                xserv.refresh(w);
            }
        }
    }

    fn special(&mut self, dir: Direction){
        match dir {
            Direction::Forward => if self.columns < 5 {
                self.columns+=1;
            },
            Direction::Backward => if self.columns > 0 {
                self.columns-=1;
            },
            _ => ()
        }
    }

    fn add(&mut self, wind: &mut Window){
        println!("Added a new window, yaaay");
    }
    fn remove(&mut self, ind: usize){
        println!("Removed window {}, oh nooo",ind);
    }
}
