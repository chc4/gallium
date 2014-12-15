extern crate serialize;
extern crate xlib;
use std::os::homedir;
use std::sync::RWLock;
use serialize::{Encodable,Decodable,json, Encoder,Decoder};
use std::io::{File,Open,Truncate,ReadWrite,Reader};
use key::Key;
use xserver::XServer;
// Common configuration options for the window manager.

#[deriving(Clone)]
pub struct KeyBind {
    pub binding: Option<Key>, //Can't store the Key in this
    pub chord: String
}

impl KeyBind{
    pub fn create(s: String) -> KeyBind {
        KeyBind {
            binding: None,
            chord: s
        }
    }
    pub fn unwrap(self) -> Key {
        self.binding.unwrap()
    }
}
//When decoding JSON to Config, we need to have the JSON hold *only* the chord
impl<E, D:Decoder<E>> Decodable<D,E> for KeyBind {
    fn decode(d: &mut D) -> Result<KeyBind,E> {
        let k = KeyBind {
            binding: None,
            chord: try!(d.read_str())
        };
        Ok(k)
    }
}
//Manually implement KeyBind serialization so it saves it in key chord format
impl<E, S:Encoder<E>> Encodable<S,E> for KeyBind {
    fn encode(&self, s: &mut S) -> Result<(),E> {
        s.emit_str(self.chord.as_slice())
    }
}
#[deriving(Encodable,Decodable,Clone)]
pub struct Config {
    /// Whether focus follows mouse movements or
    /// only click events and keyboard movements.
    pub focus_follows_mouse: bool,
    /// Border color for focused windows.
    pub focus_border_color: u32,
    /// Border color for unfocused windows.
    pub border_color: u32,
    /// Border width. This is the same for both, focused and unfocused.
    pub border_width: u32,
    /// Default spacing between windows
    pub spacing: u32,
    /// Default terminal to start
    pub terminal: (String, String),
    /// Path to the logfile
    pub logfile: String,
    /// Default tags for workspaces
    pub tags: Vec<String>,
    /// Default launcher application
    pub launcher: String,
    /// Keybind for the launcher and configuration reloading
    pub launch_key: KeyBind,
    /// Keybind for reloading this struct from a JSON file
    pub reload_key: KeyBind,
    /// Keybind for launching the terminal
    pub term_key: KeyBind,
    /// Keybind for cycling through the layout format
    pub mode_key: KeyBind,
    /// Modifier prefix for workspace hotkeys. Eg, M- will register M-1, M-2,...
    pub workspace_mod: String,
    /// Hotkeys for switching left and right from the current workspace
    pub workspace_left: KeyBind,
    pub workspace_right: KeyBind
}
pub struct ConfigLock {
    conf: RWLock<Config>
}

impl ConfigLock {
    pub fn current(&self) -> Config {
        self.conf.read().deref().clone()
    }
    pub fn update(&mut self,new_conf: Config){
        self.conf = RWLock::new(new_conf);
    }
    pub fn setup(&mut self,serv: &mut XServer){
        self.conf.write().setup(serv);
    }
}

/*
You have a config
You can pass window.dostuff(blah,blah,&config.current()) (a copy of the config
at that point in time)
You can also do config.update(new_config) on a keybind, so everything
is super hotswappable and happy. Yaay.
*/

impl Config {
    /// Create the Config from a json file
    pub fn initialize() -> Config {
        //Default version of the config, for fallback
        let conf = Config {
                    focus_follows_mouse: true,
                    focus_border_color:  0x00B6FFB0,
                    border_color:        0x00FFB6B0,
                    border_width:        2,
                    spacing:             10,
                    terminal:            (String::from_str("xterm"), String::from_str("-fg White -bg Black")),
                    logfile:             format!("{}/.wtftw.log", homedir().unwrap().to_c_str()),
                    tags:                vec!(
                                            String::from_str("1: term"),
                                            String::from_str("2: web"),
                                            String::from_str("3: code"),
                                            String::from_str("4: media")),
                    launcher: "dmenu".to_string(),
                    launch_key: KeyBind::create("M-S-p".to_string()), //Stop. Before you say something about using a char instead, Json::decode fails for them
                    reload_key: KeyBind::create("M-p".to_string()),
                    term_key: KeyBind::create("S-M-Return".to_string()),
                    mode_key: KeyBind::create("M-Return".to_string()),
                    workspace_mod: "M-".to_string(),
                    workspace_left: KeyBind::create("M4-Left".to_string()),
                    workspace_right: KeyBind::create("M4-Right".to_string())
        };
        let path = Path::new(format!("{}/.wtftwrc", homedir().unwrap().to_c_str()));
        let mut conf_file = File::open_mode(&path,Open,ReadWrite).unwrap();
        let dec_conf = match json::decode(conf_file.read_to_string().unwrap().as_slice()) {
            Ok(v) => v,
            Err(_) =>{
                        println!("Our config is corrupted!");
                        //Let's just roll back to the default
                        //conf_file.truncate(0);
                        let mut conf_file = File::open_mode(&path,Truncate,ReadWrite).unwrap();
                        conf_file.write_str(json::encode::<Config>(&conf).as_slice());
                        conf_file.fsync();
                        conf
                     }
        };
        dec_conf
    }
    pub fn setup(&mut self, serv: &mut XServer){
        serv.clear_keys();
        //Register the workspace hotkeys
        let numkey = ["1","2","3","4","5","6","7","8","9","0"];
        for i in range(0,numkey.len()){
            let mut mo = self.workspace_mod.clone();
            mo.push_str(numkey[i]);
            let k = Key::create(mo,serv);
            serv.add_key(k);
        }
        serv.parse_key(&mut self.launch_key);
        serv.parse_key(&mut self.reload_key);
        serv.parse_key(&mut self.term_key);
        serv.parse_key(&mut self.mode_key);
        serv.parse_key(&mut self.workspace_left);
        serv.parse_key(&mut self.workspace_right);
   }
    //Wrap a config in a RWLock
    pub fn new() -> ConfigLock {
        ConfigLock {
            conf: RWLock::new(Config::initialize())
        }
    }
}
