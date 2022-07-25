use std::io::Cursor;

pub trait Savable {
    fn save_state(&self, state: &mut Vec<u8>);
    fn load_state(&mut self, state: &mut Cursor<Vec<u8>>);
}
