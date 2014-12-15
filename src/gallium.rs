#![feature(globs,phase)]
extern crate serialize;
extern crate xlib;
#[phase(plugin,link)]
extern crate log;
use self::xlib::{
    KeyCode,
    KeySym,
    Display
};
use config::{Config,ConfigLock};
use window_manager::WindowManager;
use xserver::XServer;

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
                _ => {
                    println!("Fetched event");
                }
            }
        }
    }
}

fn main(){
    let gl = Gallium::setup();
    debug!("Gallium has been setup.");
    gl.start();
}
