use crate::data::deco::Decoration;

pub struct CalcDeco<'a> {
    pub base: &'a Decoration,
}

impl<'a> CalcDeco<'a> {
    pub fn new(base: &'a Decoration) -> Self {
        Self { base }
    }
}
