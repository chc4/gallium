extern crate xlib;
use std::os::homedir;
use std::sync::RwLock;
use rustc_serialize::{Encodable,Decodable,json,Encoder,Decoder};
use std::old_io::{File,Open,Truncate,ReadWrite,Reader};
use key::Key;
use xserver::XServer;
use std::ffi::CString;
// Common configuration options for the window manager.

#[derive(Clone)]
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
    pub fn unwrap(&self) -> &Key {
        self.binding.as_ref().unwrap()
    }
}
//When decoding JSON to Config, we need to have the JSON hold *only* the chord
impl Decodable for KeyBind {
    fn decode<D: Decoder>(d: &mut D) -> Result<KeyBind,D::Error> {
        let k = KeyBind {
            binding: None,
            chord: try!(d.read_str())
        };
        Ok(k)
    }
}
//Manually implement KeyBind serialization so it saves it in key chord format
impl Encodable for KeyBind {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(),S::Error> {
        s.emit_str(&self.chord[..])
    }
}
#[derive(RustcEncodable,RustcDecodable,Clone)]
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
    /// Default tags for workspaces
    pub tags: Vec<String>,
    /// Modifier for WM keybinds
    pub kommand: KeyBind,
    /// Default launcher application
    pub launcher: String,
    /// Keybind for the launcher and configuration reloading
    pub launch_key: KeyBind,
    /// Keybind for reloading this struct from a JSON file
    pub reload_key: KeyBind,
    /// Keybind for launching the terminal
    pub term_key: KeyBind,
    /// Hotkeys for switching left and right from the current workspace
    pub workspace_left: KeyBind,
    pub workspace_right: KeyBind
}
pub struct ConfigLock {
    conf: RwLock<Config>
}

impl ConfigLock {
    pub fn current(&self) -> Config {
        self.conf.read().unwrap().clone()
    }
    pub fn update(&mut self,new_conf: Config){
        self.conf = RwLock::new(new_conf);
    }
    pub fn setup(&mut self,serv: &mut XServer){
        self.conf.write().unwrap().setup(serv);
    }
}

/*
You have a config
You can pass window.dostuff(blah,blah,&config.current()) (a copy of the config
at that point in time)
You can also do config.update(new_config) on a keybind, so everything
is super hotswappable and happy. Yaay.
*/
fn default() -> Config {
    Config {
        focus_follows_mouse: true,
        focus_border_color:  0x00B6FFB0,
        border_color:        0x00FFB6B0,
        border_width:        2,
        spacing:             10,
        terminal:            ("urxvt".to_string(), "".to_string()),
        tags:                vec!(
            "1: term".to_string(),
            "2: web".to_string(),
            "3: code".to_string(),
            "4: media".to_string()),
        kommand: KeyBind::create("M4-".to_string()),
        launcher: "dmenu".to_string(),
        launch_key: KeyBind::create("K-S-p".to_string()),
        reload_key: KeyBind::create("K-p".to_string()),
        term_key: KeyBind::create("S-K-Return".to_string()),
        workspace_left: KeyBind::create("K-Left".to_string()),
        workspace_right: KeyBind::create("K-Right".to_string())
    }
}

impl Config {
    /// Create the Config from a json file
    pub fn initialize() -> Config {
        let path = Path::new(CString::from_slice(format!("{}/.galliumrc", homedir().unwrap().as_str().unwrap()).as_bytes()));
        let mut conf_file = File::open_mode(&path,Open,ReadWrite).unwrap();
        let dec_conf = match json::decode(&conf_file.read_to_string().unwrap()[..]) {
            Ok(v) => v,
            Err(e) =>{
                        println!("Our config is corrupted!");
                        println!("Error: {:?}",e);
                        default()
                     }
        };
        dec_conf
    }
    pub fn setup(&mut self, serv: &mut XServer){
        serv.clear_keys();
        //Instance Kommand to a valid Key
        let mut kchord = self.kommand.chord.clone();
        kchord.push_str("A"); //Make it a parsable KeyBind
        self.kommand.binding = Some(Key::create(kchord,serv));
        //...And then add kontrol to the XServer cell (hacky!)
        *serv.kommand_mod.borrow_mut() = Some(self.kommand.unwrap().modifier.clone());
 
        //Register the workspace hotkeys
        let numkey = ["1","2","3","4","5","6","7","8","9","0"];
        for num in numkey.iter() {
            let mut k = Key::create(num.to_string(),serv);
            k.modifier = k.modifier | self.kommand.unwrap().modifier;
            serv.add_key(k);
        }
        let mut parse = vec!(
            &mut self.launch_key,
            &mut self.reload_key,
            &mut self.term_key,
            &mut self.workspace_left,
            &mut self.workspace_right);
        for k in parse.iter_mut() {
            let p = Key::create(k.chord.clone(),serv);
            k.binding = Some(p);
            serv.add_key(p);
        }
        serv.grab_keys();
    }

    pub fn reset(&mut self){
        //Let's just roll back to the default
        //conf_file.truncate(0);
        let path = Path::new(CString::from_slice(format!("{}/.galliumrc", homedir().unwrap().as_str().unwrap()).as_bytes()));
        let mut conf_file = File::open_mode(&path,Truncate,ReadWrite).unwrap();
        let mut buff = String::new();
        {
            let mut pretty = json::Encoder::new_pretty(&mut buff);
            (&default()).encode(&mut pretty).unwrap();
        }
        //conf_file.write_str(&json::encode::<Config>(&default()).unwrap()[..]);
        conf_file.write_str(&buff[..]);
        conf_file.fsync();
    }

    //Wrap a config in a RWLock
    pub fn new() -> ConfigLock {
        ConfigLock {
            conf: RwLock::new(Config::initialize())
        }
    }
}
