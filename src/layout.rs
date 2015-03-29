use xserver::{XServer};
use super::Gallium;
use window_manager::{Deck,Workspace,Window};
use config::{Config,Direction};

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
    fn special_add(&mut self, Option<usize>){
        println!("Layout.add_special unimplemented");
    }
    fn special_sub(&mut self, Option<usize>){
        println!("Layout.remove_special unimplemented");
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
    fn special_add(&mut self, s: Option<usize>){
        panic!("Called Layout.add_special on placeholder!");
    }
    fn special_sub(&mut self, s: Option<usize>){
        panic!("Called Layout.remove_special on placeholder!");
    }
}

pub struct TallLayout {
    pub columns: u16, //How many columns there are
    pub master: Vec<u32>
}

impl Layout for TallLayout {
    fn apply(&self, screen: u32, xserv: &mut XServer, work: &mut Workspace, config: &mut Config){
        let mut wind = &mut work.windows.cards[..];
        let (x,y) = (xserv.width(screen as u32) as usize,xserv.height(screen as u32) as usize);

        let mut col = if self.columns as usize >= wind.len() {
            (wind.len()) as usize
        } else {
            self.columns as usize 
        };
        if col == 0 {
            col = 1
        }
        println!("Col: {}",col);
        let mcol = (col-1); //I use this a lot
        for c in 0..mcol {
            println!("Applying layout to window {} (master)",c);
            let ref mut w = wind[c];
            w.x = ((x/col)*c) as isize;
            w.y = 0;
            w.size = ((x/col) as usize,y as usize);
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
    
    fn special_add(&mut self, ind: Option<usize>){
        if self.columns < 5 {
            self.columns+=1;
        }
    }
    
    fn special_sub(&mut self, ind: Option<usize>){
        if self.columns > 1 {
            self.columns-=1;
        }
    }

    fn add(&mut self, wind: &mut Window){
        println!("Added a new window, yaaay");
    }
    fn remove(&mut self, ind: usize){
        println!("Removed window {}, oh nooo",ind);
    }
}
