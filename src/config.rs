extern crate xlib;
extern crate serialize;
use std::os::homedir;
use std::sync::RwLock;
use rustc_serialize::{Encodable,Decodable,json,Encoder,Decoder};
use serialize::Decoder as StdDecoder;
use std::old_io::{File,Open,Truncate,ReadWrite,Reader,Writer};
use std::old_path::{Path,GenericPath};
use key::Key;
use xserver::XServer;
use std::ffi::CString;
use self::Direction::*;
// Common configuration options for the window manager.

#[derive(Clone,RustcDecodable,RustcEncodable)]
pub enum Direction {
    Backward,
    Forward,
    Up, //Generally unused, but if I ever want 2d-array workspaces...
    Down
}
#[derive(Clone,RustcDecodable,RustcEncodable)]
pub enum Message {
    Spawn(String,String),
    Terminal,
    Reload,
    Quit,
    Kill,
    Shrink,
    Grow,
    Focus(Direction),
    Translate(Direction),
    Switch(Direction),
    Bring(Direction),
    Master,
    Special(Direction),
    None,
}

#[derive(Clone)]
pub struct KeyBind {
    pub binding: Option<Key>, //Can't store the Key in this
    pub chord: String,
    pub message: Message
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
#[derive(RustcEncodable,RustcDecodable,Clone)]
pub struct Config {
    kommand: KeyBind,
    pub terminal: (String,String),
    pub padding: u16,
    pub border: u16,
    pub spacing: u16,
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
*/
fn default() -> Config {
    Config {
        kommand: KeyBind::new("M4-",Message::None),
        padding: 5,
        border: 3,
        spacing: 5,
        terminal: ("urxvt".to_string(), "".to_string()),
        keys: vec!(
            KeyBind::new("K-S-Return",Message::Terminal),
            KeyBind::new("K-q",Message::Reload),
            KeyBind::new("K-S-q",Message::Quit),
            KeyBind::new("K-x",Message::Kill),
            KeyBind::new("K-j",Message::Shrink),
            KeyBind::new("K-semicolon",Message::Grow),
            KeyBind::new("K-k",Message::Focus(Forward)),
            KeyBind::new("K-l",Message::Focus(Backward)),
            KeyBind::new("K-S-k",Message::Translate(Forward)),
            KeyBind::new("K-S-l",Message::Translate(Backward)),

            KeyBind::new("K-Right",Message::Switch(Forward)),
            KeyBind::new("K-Left",Message::Switch(Backward)),
            KeyBind::new("K-S-Right",Message::Bring(Forward)),
            KeyBind::new("K-S-Left",Message::Bring(Backward)),

            KeyBind::new("K-Return",Message::Master),
            KeyBind::new("K-comma",Message::Special(Backward)),
            KeyBind::new("K-period",Message::Special(Forward)),
        ),
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
