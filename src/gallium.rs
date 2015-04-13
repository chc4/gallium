#![feature(rustc_private,os,old_path,std_misc,old_io,collections,libc,core)]
#[macro_use] extern crate log;
extern crate rustc_serialize;
#[macro_use] extern crate rustc_bitflags;
extern crate serialize;
extern crate xlib;
extern crate libc;
use config::{Message,Config,ConfigLock,Direction};
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
                    let screen = self.window_manager.screens.index.unwrap() as u32;
                    let p = window.wind_ptr;
                    w.layout.add(&mut window);
                    w.windows.push(window);
                    self.window_server.map(p);
                    w.refresh(&mut self.window_server, screen, self.config.current());
                },
                ServerEvent::KeyPress(key) => {
                    self.dispatch_key(key);
                },
                ServerEvent::DestroyNotify(wind_ptr) => {
                    let screen = self.window_manager.screens.index.unwrap() as u32;
                    for mut work in &mut self.window_manager.workspaces.cards[..] {
                        for wind_ind in 0..work.windows.cards.len() {
                            if work.windows.cards[wind_ind].wind_ptr == wind_ptr {
                                println!("Window {} removed from Workspace",wind_ind);
                                work.layout.remove(wind_ind);
                                work.windows.remove(wind_ind);
                                work.refresh(&mut self.window_server, screen, self.config.current());
                                break;
                            }
                        }
                    }
                },
                _ => {
                    println!("Fetched event");
                }
            }
        }
    }

    fn dispatch_key(&mut self, key: Key){
        println!("Key press:{:?}",key);
        //TODO: figure out how I am going to handle multi-screen
        let screen = self.window_manager.screens.index.unwrap() as u32;
        for k in self.config.current().keys {
            if k.binding.unwrap() == key {
                match k.message {
                    Message::Terminal => {
                        println!("Spawning terminal...");
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
                    },
                    Message::Reload => {
                        self.window_server.clear_keys();
                        let mut new_conf = Config::new();
                        new_conf.setup(&mut self.window_server);
                        self.config = new_conf;
                        println!("Config reloaded OK!");
                    },
                    Message::Quit => {
                        unsafe {
                            self.window_server.quit(k.binding.unwrap());
                            println!("Goodbye!");
                            libc::exit(0);
                        }
                    },
                    Message::Kill => {
                        let mut work = self.window_manager.workspaces.current().unwrap();
                        {
                            let mut wind = work.windows.current();
                            if wind.is_some() {
                                unsafe {
                                    self.window_server.kill_window(wind.unwrap().wind_ptr);
                                    println!("Killed window");
                                }
                            }
                        }
                        let wind_ind = work.windows.index;
                        if wind_ind.is_some() {
                            work.windows.forget(wind_ind.unwrap());
                        }
                        work.refresh(&mut self.window_server, screen, self.config.current());
                    },
                    Message::Focus(dir) => {
                        let mut work = self.window_manager.workspaces.current().unwrap();
                        match dir {
                            Direction::Backward => {
                                let i = work.windows.index.unwrap_or(1);
                                let l = work.windows.cards.len();
                                if l == 0 {
                                    work.windows.index = None;
                                }
                                else if i == 0 {
                                    work.windows.index = Some(l-1);
                                }
                                else {
                                    work.windows.index = Some(i-1);
                                }
                            },
                            Direction::Forward => {
                                let i = work.windows.index.unwrap_or(0);
                                let l = work.windows.cards.len();
                                if l == 0 {
                                    work.windows.index = None;
                                }
                                else if i == (l-1) {
                                    work.windows.index = Some(0);
                                }
                                else {
                                    work.windows.index = Some(i+1);
                                }
                            },
                            _ => ()
                        }
                        work.refresh(&mut self.window_server, screen, self.config.current());
                    },
                    Message::Special(dir) => {
                        let mut work = self.window_manager.workspaces.current().unwrap();
                        work.layout.special(dir);
                        work.refresh(&mut self.window_server, screen, self.config.current());
                    },
                    Message::None => (),
                    _ => println!("Unknown key message!")
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
            return;
        }
    }
    debug!("Gallium has been setup.");
    gl.start();
}
