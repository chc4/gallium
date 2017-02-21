//A Vec<T> that has a `current` member.
#[derive(PartialEq)]
pub struct Deck<T>{
    pub cards: Vec<T>, //If you get weird mutability errors, RefCell<T>
    pub index: Option<usize>
}

impl<T> Deck<T>{
    pub fn new() -> Deck<T>{
        Deck {
            cards: Vec::new(),
            index: None
        }
    }

    pub fn select(&mut self, ind: usize){
        if ind >= self.cards.len() {
            panic!("Selected a card index that doesn't exist");
        }
        self.index = Some(ind)
    }
    pub fn current(&mut self) -> Option<&mut T>{
        if self.index.is_none() || self.index.unwrap() >= self.cards.len() {
            return None
        }
        Some(&mut self.cards[self.index.unwrap()])
    }

    pub fn push(&mut self,card: T){
        self.cards.push(card);
        if self.index.is_none() {
            self.index = Some(self.cards.len()-1);
        }
    }
    pub fn pop(&mut self) -> Option<T>{
        let r = self.cards.pop();
        if self.index.is_some() && self.cards.len() >= self.index.unwrap() {
            self.index = None;
        }
        Some(r.unwrap())
    }
    pub fn swap(&mut self,pos1: usize,pos2: usize){
        self.cards.swap(pos1,pos2);
    }
    //This is O(n) and re-allocates everything right of the index. It's bad.
    pub fn remove(&mut self,ind: usize) -> Option<T> {
        let k = self.cards.remove(ind);
        // If we can avoid nuking index, then just select the next one in line
        if self.index.is_some() && self.index.unwrap() >= ind && self.cards.len() <= self.index.unwrap()  {
            self.index = None
        }
        Some(k)
    }
    //Remove the element at index, but don't move self.index from it
    //If the index is None or would be OoB, return false
    pub fn forget(&mut self,ind: usize) -> bool {
        let old = self.index.clone();
        let elem = self.remove(ind);
        if old.is_some() && self.index.is_none() {
            if old.unwrap()<self.cards.len() {
                self.select(old.unwrap());
                return true
            }
        }
        false
    }

    fn slice(&mut self) -> &[T] {
        &self.cards[..]
    }
}
