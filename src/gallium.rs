#![feature(rustc_private,os,old_path,std_misc,old_io,collections)]
#[macro_use] extern crate log;
extern crate "rustc-serialize" as rustc_serialize;
#[macro_use] extern crate rustc_bitflags;
extern crate serialize;
extern crate xlib;
use config::{Config,ConfigLock};
use window_manager::WindowManager;
use xserver::{XServer,ServerEvent};
use key::Key;
use std::process::Command;

pub mod config;
pub mod window_manager;
pub mod key;
pub mod xserver;
pub mod layout;

pub struct Gallium<'a> {
    config: ConfigLock,
    window_manager: WindowManager<'a>,
    window_server: XServer
}

impl<'a> Gallium<'a> {
    fn setup() -> Gallium<'a> {
        let mut window_server = unsafe { XServer::new() };
        let mut config = Config::new();
        config.setup(&mut window_server);
        let window_manager = WindowManager::new(&window_server,&config);

        Gallium {
            config: config,
            window_manager: window_manager,
            window_server: window_server
        }
    }

    fn start(mut self){
        loop {
            debug!("Polling event...");
            match self.window_server.get_event() {
                ServerEvent::MapRequest(mut window) => {
                    //For now just adding to the current workspace
                    let mut w = self.window_manager.workspaces.current().unwrap();
                    let screen = self.window_manager.screens.current.unwrap() as u32; //index, not ptr
                    let p = window.wind_ptr;
                    w.layout.add(&mut window);
                    w.windows.push(window);
                    self.window_server.map(p);
                    w.refresh(&mut self.window_server, screen, self.config.current());
                },
                ServerEvent::KeyPress(key) => {
                    println!("Key press:{:?}",key);
                    if key == *self.config.current().term_key.unwrap() {
                        println!("Should probably spawn a terminal right here");
                        let (term,args) = self.config.current().terminal.clone();
                        let (term,args) = (term.as_slice(),args.as_slice());
                        let mut c = Command::new(term);
                        if args.len()>0 {
                            let co = c.arg(args);
                            co.spawn();
                        }
                        else {
                            c.spawn();
                        }
                    }
                },
                ServerEvent::DestroyNotify(wind_ptr) => {
                    let screen = self.window_manager.screens.current.unwrap() as u32;
                    for mut work in &mut self.window_manager.workspaces.cards[..] {
                        for wind_ind in range(0,work.windows.cards.len()) {
                            if work.windows.cards[wind_ind].wind_ptr == wind_ptr {
                                println!("Window {} removed from Workspace",wind_ind);
                                work.layout.remove(wind_ind as u32);
                                work.windows.remove(wind_ind as u32);
                                work.refresh(&mut self.window_server, screen, self.config.current());
                                break;
                            }
                        }
                    }
                }
                _ => {
                    println!("Fetched event");
                }
            }
        }
    }
}

fn main(){
    let gl = Gallium::setup();
    for argument in std::os::args().iter() {
        println!("{}", argument);
        if argument[..].eq("--revert") {
            Config::reset(&mut gl.config.current());
            println!("Reverted config!");
        }
    }
    debug!("Gallium has been setup.");
    gl.start();
}
