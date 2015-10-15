extern crate xlib;
extern crate rustc_serialize;
use std::env::home_dir;
use std::sync::RwLock;
use rustc_serialize::{Encodable,Decodable,json,Encoder,Decoder};
use rustc_serialize::Decoder as StdDecoder;
use rustc_serialize::json::ParserError;
use std::io::{Error,ErrorKind,Read,Write};
use std::fs::OpenOptions;
use std::path::PathBuf;
use key::Key;
use xserver::XServer;
use layout::{LayoutFactory,Layouts};
use self::Direction::*;
// Common configuration options for the window manager.

#[derive(Clone)]
pub struct KeyBind {
    pub binding: Option<Key>,
    pub chord: String,
    pub message: Message
}

#[derive(Clone,RustcEncodable,RustcDecodable)]
pub struct WorkspaceConf {
    pub name: String,
    pub layout: Layouts,
}

impl KeyBind {
    pub fn new(s: &str, mess: Message) -> KeyBind {
        KeyBind {
            binding: None,
            chord: s.to_string(),
            message: mess
        }
    }

    pub fn unwrap(&self) -> &Key {
        self.binding.as_ref().unwrap()
    }
}
//When decoding JSON to Config, we need to have the JSON hold *only* the chord
impl Decodable for KeyBind {
    fn decode<D: Decoder>(d: &mut D) -> Result<KeyBind,D::Error> {
        d.read_tuple(2,|d: &mut D|{
            let k = KeyBind {
                binding: None,
                chord: try!(d.read_str()),
                message: try!(<Message as Decodable>::decode(d))
            };
            Ok(k)
        })
    }
}
//Manually implement KeyBind serialization so it saves it in key chord format
impl Encodable for KeyBind {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(),S::Error> {
        s.emit_tuple(2,|s: &mut S|{
            s.emit_tuple_arg(0,|s: &mut S| s.emit_str(&self.chord[..]) );
            s.emit_tuple_arg(1,|s: &mut S| self.message.encode::<S>(s) )
        })
    }
}

#[derive(Clone)]
pub struct Color(pub u32);
impl Encodable for Color {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(),S::Error> {
        s.emit_str(&format!("{:X}",self.0))
    }
}
impl Decodable for Color {
    fn decode<D: Decoder>(d: &mut D) -> Result<Color,D::Error> {
        let c = try!(
            u32::from_str_radix(&try!(d.read_str()).to_string(),16)
            .map_err(|e|
                d.error("ParseIntError")  // no clue how this should actually work
            )
        );
        Ok(Color(c))
    }
}

#[derive(RustcEncodable,RustcDecodable,Clone)]
pub struct Config {
    kommand: KeyBind,
    pub padding: u16,
    pub border: u32,
    pub spacing: u16,
    pub follow_mouse: bool,
    pub focus_color: Color,
    pub unfocus_color: Color,
    pub workspaces: Vec<WorkspaceConf>,
    pub keys: Vec<KeyBind>,
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

/*
 * Ok so problem:
 * Adding new keys is a giant pain.
 * Add a new field to Config, add a new default
 * set the new default to KeyBind::Create("new-bind")
 * add the key to the initializion list
 * (eventually) add it to the callback registering
 *
 * Solution:
 * Instead just have a Vec<T> of Keys
 * Have the Key also have a command field
 * command is an enum for what the message is
 * Command::Reload would be matched...uh, somewhere
 * and do the actual command
 *
 * (i totally didn't steal this idea from xr3wm nope i am super original)
*/#[derive(Clone,RustcDecodable,RustcEncodable)]
pub enum Direction {
    Backward,
    Forward,
    Up,
    Down
}
// Special messages, for handling/ignore with layout.special
#[derive(Clone,RustcDecodable,RustcEncodable)]
pub enum SpecialMsg {
    Grow,
    Shrink,
    Add,
    Subtract
}

#[derive(Clone,RustcDecodable,RustcEncodable)]
pub enum Message {
    Spawn(String,String),
    Reload,
    Quit,
    Kill,
    Focus(Direction),
    Translate(Direction),
    Resize(Direction),
    Move(Direction),
    Switch(Direction),
    Bring(Direction),
    Master,
    Special(SpecialMsg),
    Chain(Vec<Message>),
    None,
}

fn default() -> Config {
    Config {
        kommand: KeyBind::new("M4-",Message::None),
        // The dead-space from the borders of the screen
        padding: 5,
        border: 1,
        // The blank-space in between tiled windows
        spacing: 5,
        // If window focus should follow your mouse cursor or not
        follow_mouse: true,
        // Border color for the focused window
        focus_color: Color(0x3DAFDC),
        // ...and for unfocused ones
        unfocus_color: Color(0x282828),
        workspaces: vec!(
            WorkspaceConf { name: "|".to_string(), layout: Layouts::Tall },
            WorkspaceConf { name: "||".to_string(), layout: Layouts::Full },
        ),
        keys: vec!(
            KeyBind::new("K-S-Return",Message::Spawn("urxvt".to_string(),"".to_string())),
            KeyBind::new("K-q",Message::Reload),
            KeyBind::new("K-S-q",Message::Quit),
            KeyBind::new("K-x",Message::Kill),
            KeyBind::new("K-j",Message::Special(SpecialMsg::Shrink)),
            KeyBind::new("K-semicolon",Message::Special(SpecialMsg::Grow)),
            KeyBind::new("K-k",Message::Focus(Forward)),
            KeyBind::new("K-l",Message::Focus(Backward)),
            KeyBind::new("K-S-k",Message::Translate(Forward)),
            KeyBind::new("K-S-l",Message::Translate(Backward)),

            KeyBind::new("K-Right",Message::Switch(Forward)),
            KeyBind::new("K-Left",Message::Switch(Backward)),
            KeyBind::new("K-S-Right",Message::Bring(Forward)),
            KeyBind::new("K-S-Left",Message::Bring(Backward)),

            KeyBind::new("K-Return",Message::Master),
            KeyBind::new("K-minus",Message::Special(SpecialMsg::Subtract)),
            KeyBind::new("K-equal",Message::Special(SpecialMsg::Add)),

            // For easy rearranging of floating windows
            KeyBind::new("K-m",Message::Resize(Backward)),
            KeyBind::new("K-comma",Message::Resize(Down)),
            KeyBind::new("K-period",Message::Resize(Up)),
            KeyBind::new("K-slash",Message::Resize(Forward)),
            KeyBind::new("K-S-m",Message::Move(Backward)),
            KeyBind::new("K-S-comma",Message::Move(Down)),
            KeyBind::new("K-S-period",Message::Move(Up)),
            KeyBind::new("K-S-slash",Message::Move(Forward)),

        ),
    }
}

impl Config {
    /// Create the Config from a json file
    pub fn load(path: Option<String>) -> ConfigLock {
        let mut path = if let Some(p) = path {
            PathBuf::from(&p)
        } else {
            let mut path = home_dir().unwrap();
            path.push(".galliumrc");
            path
        };
        let mut fopt = OpenOptions::new();
        fopt.write(true).read(true);
        let mut conf_file = fopt.open(path).unwrap();
        let mut buff = String::new();
        conf_file.read_to_string(&mut buff);

        let dec_conf = match json::decode(&buff) {
            Ok(v) => v,
            Err(e) =>{
                        error!("Our config is corrupted!");
                        error!("Error: {:?}",e);
                        default()
                     }
        };
        ConfigLock {
            conf: RwLock::new(dec_conf)
        }
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
        for ref mut k in self.keys.iter_mut() {
            let p = Key::create(k.chord.clone(),serv);
            k.binding = Some(p);
            serv.add_key(p);
        }
        serv.grab_keys();
    }

    pub fn reset(&mut self){
        //Let's just roll back to the default
        //conf_file.truncate(0);
        let mut path = home_dir().unwrap();
        path.push(".galliumrc");
        let mut fopt = OpenOptions::new();
        fopt.read(true).write(true).truncate(true);
        let mut conf_file = fopt.open(path).unwrap();
        let mut buff = String::new();
        {
            let mut pretty = json::Encoder::new_pretty(&mut buff);
            (&default()).encode(&mut pretty).unwrap()
        }
        conf_file.write(&buff.into_bytes());
        conf_file.sync_data();
    }

    //Wrap a config in a RWLock
    pub fn new() -> ConfigLock {
        ConfigLock {
            conf: RwLock::new(default())
            //conf: RwLock::new(Config::initialize())
        }
    }
}
