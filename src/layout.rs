use xserver::{XServer,Window};
use super::Gallium;
use window_manager::WindowManager;
use config::Config;

pub trait Layout {
    fn apply(&mut self, &mut XServer, &mut WindowManager, &mut Config);
    fn resize(&mut self, Resize);
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
    Shrink_Horz,
    Shrink_Vert,
    Grow_Horz,
    Grow_Vert
}

pub struct TallLayout {
    pub splits: u16, //How many columns there are
    pub master: Vec<u32>
}

impl Layout for TallLayout {
    fn apply(&mut self, xserv: &mut XServer, window_manager: &mut WindowManager, config: &mut Config){
        let mut wind = &mut window_manager.workspaces.current().unwrap().windows.cards[..];
        let screen = window_manager.screens.current.unwrap();
        let (x,y) = (xserv.width(screen as u32),xserv.height(screen as u32));

        let col = if self.splits as usize >= wind.len() {
            (wind.len()) as u32
        } else {
            self.splits as u32
        };
        for c in range(0,col) {
            println!("Applying layout to window {}",c);
            let ref mut w = wind[c as usize];
            w.x = ((x/col)*c) as isize;
            w.y = y as isize;
            w.size = ((x/col) as isize,y as isize);
            xserv.refresh(w);
        }
    }
    
    fn resize(&mut self, message: Resize){
        // lol ignore horz it's tall layout remember
        match message {
            Resize::Shrink_Horz => {
                self.splits-=1;
            },
            Resize::Grow_Horz => {
                self.splits+=1;
            },
            _ => ()
        }
    }
}
