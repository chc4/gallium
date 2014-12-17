use std::fmt::{Show,Formatter,Result};
use xlib::{KeyCode,KeySym};
use xserver::{XServer,keysym_to_string};

pub bitflags! {
    #[allow(non_upper_case_globals)]
    #[deriving(Copy)]
    flags KeyMod: u32 {
        const Shift = 0b00000001,
        const Lock = 0b00000010,
        const Control = 0b00000100,
        const Mod1 = 0b00001000,
        const Mod2 = 0b00010000,
        const Mod3 = 0b00100000,
        const Mod4 = 0b01000000,
        const Mod5 = 0b10000000,
    }
}

#[deriving(PartialEq,Eq,Clone,Copy)]
pub struct Key {
    pub code: KeyCode,
    pub sym: KeySym,
    pub modifier: KeyMod,
}
const LOOKUP: [&'static str,..8] = ["S-","Lock-","C-","M-","M2-","M3-","M4-","M5-"];
impl Show for Key {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mut pref = "".to_string();
        for b in range(0,8){
            if self.modifier.bits() & (1<<b) == 1<<b {
                pref.push_str(LOOKUP[b]);
            }
        }
        let cs = keysym_to_string(self.sym);
        let x = cs.as_str().unwrap();
        write!(f,"{}{}",pref,x)
    }
}

impl Key {
    pub fn parse(code: KeyCode, modifier: u32, serv: &XServer) -> Key {
        let mut sym = 0;
        unsafe {
            sym = serv.keycode_to_keysym(code);
        }
        let mo = match KeyMod::from_bits(modifier) {
            Some(v) => v,
            None => KeyMod::empty()
        };
        Key {
            code: code,
            sym: sym,
            modifier: mo
        }
    } 

    pub fn create(s: String, serv: &XServer) -> Key {
        let mut flag = 0u32;
        let mut key = "";
        for m in s.split('-') {
            let mut full = m.to_string();
            full.push_str("-");
            match LOOKUP.position_elem(&full.as_slice()) {
                Some(v) => {
                    flag = flag | 1<<v;
                },
                None => {
                    key = m;
                }
            }
        }
        let sym = serv.string_to_keysym(&key.to_string());
        let code = serv.keysym_to_keycode(sym);

        let flag = match KeyMod::from_bits(flag){
            Some(v) => v,
            None => KeyMod::empty()
        };
        Key {
            code: code as KeyCode,
            sym: sym,
            modifier: flag
        }
    }
}
