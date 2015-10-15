#![feature(collections,libc,core,slice_position_elem)]
#[macro_use] extern crate log;
extern crate rustc_serialize;
#[macro_use] extern crate bitflags;
extern crate xlib;
extern crate libc;
use config::{Message,Config,ConfigLock,Direction};
use window_manager::{WindowManager,Cycle};
use xserver::{XServer,ServerEvent};
use key::Key;
use std::process::Command;
use gallium_log::GalliumLog;

pub mod config;
pub mod window_manager;
pub mod key;
pub mod xserver;
pub mod layout;
pub mod gallium_log;

pub struct Gallium<'a> {
    config: ConfigLock,
    window_manager: WindowManager<'a>,
    window_server: XServer
}

impl<'a> Gallium<'a> {
    fn setup() -> Gallium<'a> {
        let mut window_server = unsafe { XServer::new() };
        let mut config = Config::new();
        let window_manager = WindowManager::new(&window_server,&config);

        Gallium {
            config: config,
            window_manager: window_manager,
            window_server: window_server
        }
    }

    fn start(mut self){
        loop {
            trace!("Polling event...");
            match self.window_server.get_event() {
                ServerEvent::MapRequest(mut window) => {
                    //For now just adding to the current workspace
                    let mut w = self.window_manager.workspaces.current().unwrap();
                    let screen = self.window_manager.screens.index.unwrap() as u32;
                    let p = window.wind_ptr;
                    let conf = self.config.current();
                    w.layout.add(&mut window, &mut self.window_server);
                    w.windows.push(window);
                    self.window_server.set_border_width(p,conf.border);
                    w.refresh(&mut self.window_server, screen, conf);
                },
                ServerEvent::KeyPress(key) => {
                    self.dispatch_key(key);
                },
                ServerEvent::DestroyNotify(wind_ptr) => {
                    let screen = self.window_manager.screens.index.unwrap() as u32;
                    for mut work in &mut self.window_manager.workspaces.cards[..] {
                        for wind_ind in 0..work.windows.cards.len() {
                            if work.windows.cards[wind_ind].wind_ptr == wind_ptr {
                                debug!("Window {} removed from Workspace",wind_ind);
                                work.layout.remove(wind_ind, &mut self.window_server);
                                work.windows.remove(wind_ind);
                                work.refresh(&mut self.window_server, screen, self.config.current());
                                break;
                            }
                        }
                    }
                },
                ServerEvent::EnterNotify(wind_ptr) => {
                    let conf = self.config.current();
                    if conf.follow_mouse == false {
                        continue;
                    }
                    let screen = self.window_manager.screens.index.unwrap() as u32;
                    let mut work = self.window_manager.workspaces.current().unwrap();
                    for wind_ind in 0..work.windows.cards.len() {
                        if work.windows.cards[wind_ind].wind_ptr == wind_ptr {
                            work.windows.select(wind_ind);
                            work.refresh(&mut self.window_server, screen, conf);
                            break;
                        }
                    }
                },
                _ => {
                    trace!("Unknown event");
                }
            }
        }
    }

    fn dispatch_key(&mut self, key: Key){
        debug!("Key press: {:?}",key);
        //TODO: figure out how I am going to handle multi-screen
        let screen = self.window_manager.screens.index.unwrap() as u32;
        for k in self.config.current().keys {
            if k.binding.unwrap() == key {
                match k.message {
                    Message::Spawn(exe,args) => {
                        info!("Spawning...");
                        let mut c = Command::new(exe);
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
                        info!("Config reloaded OK!");
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
                                    info!("Killed window");
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
                                let cycle = Cycle::new(l);
                                work.windows.index = cycle[(i as isize) - 1];
                            },
                            Direction::Forward => {
                                let i = work.windows.index.unwrap_or(0);
                                let l = work.windows.cards.len();
                                let cycle = Cycle::new(l);
                                work.windows.index = cycle[(i as isize) + 1];
                            },
                            _ => ()
                        }
                        work.refresh(&mut self.window_server, screen, self.config.current());
                    },
                    Message::Switch(dir) => {
                        let w_l = self.window_manager.workspaces.cards.len();
                        let w_i = self.window_manager.workspaces.index.unwrap();
                        let cycle = Cycle::new(w_l);
                        match dir {
                            Direction::Backward => {
                                self.window_manager.switch(&mut self.window_server,
                                                           self.config.current(),
                                                           cycle[(w_i as isize) - 1].unwrap() as u32);
                            },
                            Direction::Forward => {
                                self.window_manager.switch(&mut self.window_server,
                                                           self.config.current(),
                                                           cycle[(w_i as isize) + 1].unwrap() as u32);
                            },
                            _ => ()
                        }
                    },
                    Message::Special(msg) => {
                        let mut work = self.window_manager.workspaces.current().unwrap();
                        work.layout.special(msg);
                        work.refresh(&mut self.window_server, screen, self.config.current());
                    },
                    Message::None => (),
                    _ => warn!("Unknown key message!")
                }
            }
        }
    }
}

fn main(){
    GalliumLog::init();
    let mut gl = Gallium::setup();

    let mut init = false;
    for argument in std::env::args() {
        trace!("{}", argument);
        if argument[..].eq("--new") {
            Config::reset(&mut gl.config.current());
            init = true;
            info!("New config installed!");
            return;
        }
        if argument[0..3].eq("-c=") {
            println!("Using {:?} as config path",&argument[3..]);
            let mut new_conf = Config::load(Some(argument[3..].to_string()));
            new_conf.setup(&mut gl.window_server);
            gl.config = new_conf;
            init = true;
        }
    }
    if init == false {
        let mut new_conf = Config::load(None);
        new_conf.setup(&mut gl.window_server);
        gl.config = new_conf;
    }
    info!("Gallium has been setup.");
    gl.start();
}
