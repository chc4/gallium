use xserver::{XServer};
use super::Gallium;
use window_manager::{Deck,Workspace,Window};
use config::Config;

pub trait Layout {
    fn apply(&self, &mut XServer, screen: u32, &mut Deck<Window>, &mut Config);
    fn resize(&mut self, Resize);
    fn add(&mut self, &mut Window);
    fn remove(&mut self, usize);
    fn add_master(&mut self, usize);
    fn remove_master(&mut self, usize);
}

#[derive(Copy,PartialEq)]
pub enum Layouts {
    Tall,
    Wide,
    Grid,
    Stacking,
    Full
}

pub enum Resize {
    ShrinkHorz,
    ShrinkVert,
    GrowHorz,
    GrowVert
}

pub struct TallLayout {
    pub columns: u16, //How many columns there are
    pub master: Vec<u32>
}

impl Layout for TallLayout {
    fn apply(&self, xserv: &mut XServer, screen: u32, windows: &mut Deck<Window>, config: &mut Config){
        let mut wind = &mut windows.cards[..];
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
        for c in range(0,mcol) {
            println!("Applying layout to window {} (master)",c);
            let ref mut w = wind[c];
            w.x = ((x/col)*c) as isize;
            w.y = 0;
            w.size = ((x/col) as usize,y as usize);
            xserv.refresh(w);
        }
        if wind.len()-mcol as usize > 0 {
            let stack = wind.len()-mcol as usize;
            for r in range(mcol,wind.len()) {
                println!("Appling layout to window {}",r);
                let ref mut w = wind[r as usize];
                w.x = ((x/col)*mcol) as isize;
                w.y = ((y/(stack))*(r-mcol)) as isize;
                w.size = ((x/col) as usize,(y/stack) as usize);
                xserv.refresh(w);
            }
        }
    }
    
    fn add_master(&mut self, ind: usize){
        
    }
    
    fn remove_master(&mut self, ind: usize){
    
    }

    fn add(&mut self, wind: &mut Window){
        println!("Added a new window, yaaay");
    }
    fn remove(&mut self, ind: usize){
        println!("Removed window {}, oh nooo",ind);
    }

    fn resize(&mut self, message: Resize){
        // lol ignore horz it's tall layout remember
        match message {
            Resize::ShrinkHorz => {
                self.columns-=1;
            },
            Resize::GrowHorz => {
                self.columns+=1;
            },
            _ => ()
        }
    }
}
