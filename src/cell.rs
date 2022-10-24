use std::{fmt::Display, ops::Deref, rc::Rc};
use wasm_bindgen::__rt::{WasmRefCell, Ref};
use web_sys::{Text};
use crate::{Result, component::{Component, RefComponent}};

#[derive(Clone)]
pub struct SharedCell<T> {
    v: Rc<WasmRefCell<T>>,
    text: Text
}

impl<T: Display> SharedCell<T> {
    #[inline]
    pub fn new (v: T) -> Result<Self> {
        let text = Text::new_with_data(v.to_string().as_str())?;
        return Ok(Self { v: Rc::new(WasmRefCell::new(v)), text })
    }

    #[inline]
    pub fn get (&self) -> Ref<'_, T> {
        return self.v.borrow()
    }

    #[inline]
    pub fn mutate<F: FnOnce(&mut T)> (&self, f: F) {
        let mut v = self.v.borrow_mut();
        f(&mut v);
        self.redraw_inner(&v)
    }

    #[inline]
    pub fn set (&self, v: T) {
        self.mutate(|x| *x = v)
    }

    #[inline]
    pub fn update<F: FnOnce(T) -> T> (&self, f: F) {
        self.mutate(|x| unsafe {
            let v = core::ptr::read(x);
            core::ptr::write(x, f(v))
        });
    }

    #[inline]
    pub fn redraw (&self) {
        self.redraw_inner(&self.v.borrow());
    }

    #[inline]
    fn redraw_inner (&self, v: &T) {
        self.text.set_data(v.to_string().as_str())
    }
}

impl<T> Deref for SharedCell<T> {
    type Target = Text;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.text
    }
}

impl<T: Display> RefComponent for SharedCell<T> {
    #[inline]
    fn render (&self) -> Result<&web_sys::Node> {
        <web_sys::Text as RefComponent>::render(&self.text)
    }
}

impl<T: Display> Component for SharedCell<T> {
    #[inline]
    fn render (self) -> Result<web_sys::Node> {
        <web_sys::Text as Component>::render(self.text)
    }
}

#[derive(Debug)]
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
    pub fn mutate<F: FnOnce(&mut T)> (&mut self, f: F) {
        f(&mut self.v);
        self.redraw()
    }

    #[inline]
    pub fn set (&mut self, v: T) {
        self.mutate(|x| *x = v)
    }

    #[inline]
    pub fn update<F: FnOnce(T) -> T> (&mut self, f: F) {
        self.mutate(|x| unsafe {
            let v = core::ptr::read(x);
            core::ptr::write(x, f(v))
        });
    }

    #[inline]
    pub fn redraw (&self) {
        self.text.set_data(self.v.to_string().as_str())
    }
}

impl<T> Deref for Cell<T> {
    type Target = Text;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.text
    }
}

impl<T: Display> RefComponent for Cell<T> {
    #[inline]
    fn render (&self) -> Result<&web_sys::Node> {
        <web_sys::Text as RefComponent>::render(&self.text)
    }
}

impl<T: Display> Component for Cell<T> {
    #[inline]
    fn render (self) -> Result<web_sys::Node> {
        <web_sys::Text as Component>::render(self.text)
    }
}