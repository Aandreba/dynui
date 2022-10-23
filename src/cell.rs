use std::{fmt::Display, ptr::addr_of_mut, ops::Deref};
use web_sys::{Text};
use crate::Result;

pub struct Cell<T> {
    v: T,
    text: Text
}

impl<T: Display> Cell<T> {
    #[inline]
    pub fn new (v: T) -> Result<Self> {
        let text = Text::new_with_data(v.to_string().as_str())?;
        return Ok(Self { v, text })
    }

    #[inline]
    pub fn get (&self) -> &T {
        return &self.v
    }

    #[inline]
    pub fn set (&mut self, v: T) {
        self.v = v;
        self.redraw()
    }

    #[inline]
    pub fn update<F: FnOnce(T) -> T> (&mut self, f: F) {
        unsafe {
            let v = core::ptr::read(&mut self.v);
            core::ptr::write(addr_of_mut!(self.v), f(v));
            self.redraw()
        }
    }

    #[inline]
    pub fn mutate<F: FnOnce(&mut T)> (&mut self, f: F) {
        f(&mut self.v);
        self.redraw()
    }

    #[inline]
    pub fn redraw (&self) {
        self.text.set_data(self.v.to_string().as_str())
    }
}

impl<T: Display> Deref for Cell<T> {
    type Target = Text;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.text
    }
}