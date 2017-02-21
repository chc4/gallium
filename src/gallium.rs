#![feature(libc)]
#[macro_use] extern crate log;
extern crate rustc_serialize;
#[macro_use] extern crate bitflags;
extern crate xlib;
extern crate libc;
use config::{Message,Config,ConfigLock,Direction};
use window_manager::{WindowManager,Workspace,Cycle};
use layout::clamp;
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
                    Message::Master => {
                        let mut work = self.window_manager.workspaces.current().unwrap();
                        //if wind.floating XRaiseWindow
                        //else cards.forget and push(?)
                        if let Some(ind) = work.windows.index {
                            if work.windows.cards.len() > 0 {
                                work.windows.cards.swap(0, ind);
                                work.windows.select(0);
                            }
                        }
                        work.refresh(&mut self.window_server, screen, self.config.current());
                    },
                    Message::Translate(dir) => {
                        //swap adjecant windows
                        //set new current
                        let mut work = self.window_manager.workspaces.current().unwrap();
                        let off = match dir {
                            Direction::Forward => 1,
                            Direction::Backward => -1,
                            _ => 0,
                        };
                        work.windows.index.map(|i|{
                            let l = work.windows.cards.len();
                            let cycle = Cycle::new(l);
                            work.windows.swap(i, cycle[(i as isize) + off].unwrap_or(i));
                            work.windows.index = cycle[(i as isize) + off];
                        });
                        work.refresh(&mut self.window_server, screen, self.config.current());
                    },
                    Message::Reload => {
                        self.window_server.clear_keys();
                        // keep old path
                        let mut new_conf = Config::load(self.config.path.clone());
                        new_conf.setup(&mut self.window_server);
                        //replace the layouts for all the ones mapped
                        //should really just replace for ones that are wrong
                        {
                            let ls = new_conf.current().workspaces;
                            let wk = &mut self.window_manager.workspaces.cards;
                            self.config = new_conf;
                            ls.iter().zip(wk).map(|(lay,work)|{
                                info!("Layout for workspace {} is {:?}",lay.name,lay.layout);
                                work.layout = layout::LayoutFactory(lay.layout);
                            });
                        }

                        let mut work = self.window_manager.workspaces.current().unwrap();
                        work.refresh(&mut self.window_server, screen, self.config.current());
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
                    Message::Tile => {
                        let mut work = self.window_manager.workspaces.current().unwrap();
                        let mut b = false;
                        {
                            let c = work.windows.current();
                            c.map(|wind| {
                                wind.floating = !wind.floating;
                                b = true;
                            });
                        }
                        if b {
                            work.refresh(&mut self.window_server, screen, self.config.current());
                        }
                    },
                    Message::Resize(dir) => {
                        let mut work = self.window_manager.workspaces.current().unwrap();
                        let mut b = false;
                        {
                            let wind = work.windows.current();
                            wind.map(|wind| {
                                if !wind.floating {
                                    return
                                }
                                let (x,y) = match dir {
                                    Direction::Backward => (-15,0),
                                    Direction::Forward => (15,0),
                                    Direction::Up => (0,-15),
                                    Direction::Down => (0,15)
                                };
                                wind.size = (
                                    clamp(0,(wind.size.0 as i32) + x,2048) as usize,
                                    clamp(0,(wind.size.1 as i32) + y,2048) as usize
                                    );
                                b = true;
                            });
                        }
                        if b {
                            work.refresh(&mut self.window_server, screen, self.config.current());
                        }
                    },
                    Message::Move(dir) => {
                        let mut work = self.window_manager.workspaces.current().unwrap();
                        let mut b = false;
                        {
                            let wind = work.windows.current();
                            wind.map(|wind| {
                                if !wind.floating {
                                    return
                                }
                                let (x,y) = match dir {
                                    Direction::Backward => (-15,0),
                                    Direction::Forward => (15,0),
                                    Direction::Up => (0,-15),
                                    Direction::Down => (0,15)
                                };
                                wind.x = wind.x + x;
                                wind.y = wind.y + y;
                                b = true;
                            });
                        }
                        if b {
                            work.refresh(&mut self.window_server, screen, self.config.current());
                        }
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

    let mut init = false;
    let mut new_conf = None;
    for argument in std::env::args() {
        trace!("{}", argument);
        if argument[..].eq("--new") {
            Config::reset();
            init = true;
            info!("New config installed!");
            return;
        }
        if argument[0..3].eq("-c=") {
            info!("Using {:?} as config path",&argument[3..]);
            new_conf = Some(Config::load(Some(argument[3..].to_string())));
            init = true;
            continue;
        }
    }
    let mut gl = Gallium::setup();
    if init == false {
        new_conf = Some(Config::load(None));
    }
    if let Some(mut new_conf) = new_conf {
        new_conf.setup(&mut gl.window_server);
        gl.config = new_conf;
    }
    info!("Gallium has been setup.");
    gl.start();
}
