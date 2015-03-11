#![feature(rustc_private,os)]
#[macro_use] extern crate log;
extern crate "rustc-serialize" as rustc_serialize;
#[macro_use] extern crate rustc_bitflags;
extern crate serialize;
extern crate xlib;
use config::{Config,ConfigLock};
use window_manager::WindowManager;
use xserver::{XServer,ServerEvent};

pub mod config;
pub mod window_manager;
pub mod key;
pub mod xserver;
pub mod layout;

struct Gallium<'a> {
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
                ServerEvent::CreateNotify(window, (x,y)) => {
                    //For now just adding to the current workspace
                    self.window_manager.workspaces.current().unwrap().windows.push(window);
                    self.window_server.map(window);
                    self.window_manager.workspaces.current().unwrap().refresh();
                },
                ServerEvent::KeyPress(key) => {
                    println!("Key press:{:?}",key);
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
            Config::reset(&mut gl.config.current())
        }
    }
    debug!("Gallium has been setup.");
    gl.start();
}
