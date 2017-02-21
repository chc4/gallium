use std::ops::Index;
pub struct Cycle {
        max: isize,
        slice: Vec<Option<usize>>
}

impl Index<isize> for Cycle {
        type Output=Option<usize>;
        fn index(&self, index: isize) -> &Option<usize> {
                println!("Index {}",index);
                if self.max == 0 { //empty interval, return &None
                    return &self.slice[0];
                }
                if index == 0 {
                    return &self.slice[0];
                }
                let b_ind = self.max + ((index.abs()%self.max) * index.abs()/index);
                let b_ind = b_ind%self.max;
                return &self.slice[b_ind as usize];
       }
}
impl Cycle {
    pub fn new(max: usize) -> Cycle {
        let mut v = Vec::new();
        for i in 0..max {
            v.push(Some(i));
        }
        v.push(None);
        Cycle {
            max: max as isize,
            slice: v
        }
    }
}


