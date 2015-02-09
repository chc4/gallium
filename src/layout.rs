use xserver::Window;
trait Layout {
    fn apply(winds: Vec<Window>, master: &Window);
    fn reset(&mut self);
    fn resize(&mut self, vert: u64, horz: u64);
}

#[derive(Copy,PartialEq)]
pub enum Layouts {
    Tall,
    Wide,
    Grid,
    Stacking,
    Full
}

struct TallLayout {
    mid_sep: u64, //In pixels the location of the seperator bar
    master: Box<Window>
}
