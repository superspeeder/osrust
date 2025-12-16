use lazy_static::lazy_static;
use x86_64::structures::paging::PageTable;

lazy_static! {
    pub static ref LAYER_4: PageTable = PageTable::new();
}

pub fn init() {
}