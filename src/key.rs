use std::fmt::{Debug,Formatter,Result};
use std::str::from_utf8;
use x11::xlib::{KeyCode,KeySym};
use xserver::{XServer,keysym_to_string};

bitflags! {
    #[allow(non_upper_case_globals)]
    flags KeyMod: u32 {
        const SHIFT   = 0b000000001,
        const LOCK    = 0b000000010,
        const CONTROL = 0b000000100,
        const MOD1    = 0b000001000,
        const MOD2    = 0b000010000,
        const MOD3    = 0b000100000,
        const MOD4    = 0b001000000,
        const MOD5    = 0b010000000,
        const KOMMAND = 0b100000000,
    }
}

#[derive(Eq,PartialEq,Clone,Copy)]
pub struct Key {
    pub code: KeyCode,
    pub sym: KeySym,
    pub modifier: KeyMod,
}
const LOOKUP: [(&'static str, KeyMod); 9] = [("S-",SHIFT),
                                             ("Lock-",LOCK),
                                             ("C-",CONTROL),
                                             ("M-",MOD1),
                                             ("M2-",MOD2),
                                             ("M3-",MOD3),
                                             ("M4-",MOD4),
                                             ("M5-",MOD5),
                                             ("K-",KOMMAND)];
impl Debug for Key {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let (pref,x) = self.chord();
        write!(f,"{}{}",pref,x)
    }
}

impl Key {
    pub fn parse(code: KeyCode, modifier: u32, serv: &XServer) -> Key {
        let mut sym = 0;
        sym = serv.keycode_to_keysym(code);
        let mut mo = match KeyMod::from_bits(modifier) {
            Some(v) => v,
            None => KeyMod::empty()
        };
        // Replace the KOMMAND bit with what it's set to
        if (mo & KOMMAND).bits() != 0 {
            println!("Replace KOMMAND pls");
            mo = mo | serv.kommand_mod.borrow().unwrap();
            mo = mo & !KOMMAND;
        }
        Key {
            code: code,
            sym: sym,
            modifier: mo
        }
    }

    pub fn create(s: String, serv: &XServer) -> Key {
        let mut flag = KeyMod::empty();
        let mut key = "";
        for mut m in s.split('-') {
            let mut full = m.to_string();
            full.push_str("-");
            if full == "K-" {
                flag = flag | serv.kommand_mod.borrow().unwrap();
            }
            else {
                let mut found = false;
                for &(c,k) in LOOKUP.iter() {
                    if full == c {
                        flag = flag | k;
                        found = true;
                    }
                }
                if !found {
                    key = m
                }
            }
        }
        let mut sym = serv.string_to_keysym(&mut key.to_string());
        let code = serv.keysym_to_keycode(sym);

        Key {
            code: code as KeyCode,
            sym: sym,
            modifier: flag
        }
    }

    pub fn chord(&self) -> (String,&'static str) {
        let mut pref = "".to_string();
        for b in 0..8 {
            if self.modifier.contains(LOOKUP[b].1) {
                pref.push_str(LOOKUP[b].0);
            }
        }
        let cs = keysym_to_string(self.sym);

        let x = from_utf8(cs.to_bytes()).unwrap();
        (pref,x)
    }
}
