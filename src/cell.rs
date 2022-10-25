use std::{hint::unreachable_unchecked, ops::Deref, rc::Rc, cell::{UnsafeCell, RefCell, Ref}};
use web_sys::DocumentFragment;
use crate::{Result, component::{RefComponent}, attr::RefAttribute, jsprintln};

/// An object that can be treated like a cell
pub trait CellLike<'a>: 'a {
    type Value: ?Sized;
    type Ref<'b>: 'b + Deref<Target = Self::Value> where 'a: 'b;
    
    fn get (&self) -> Self::Ref<'_>;
    fn on_update<F: 'a + FnMut(&Self::Value)> (&self, f: F);
    fn on_update_once<F: 'a + FnOnce(&Self::Value)> (&self, f: F);

    #[inline]
    fn map<T, F: 'a + FnMut(&Self::Value) -> T> (&self, f: F) -> MappedCell<'a, T> {
        return MappedCell::new::<Self::Value, Self, F>(self, f)
    }

    #[inline]
    fn display (&self) -> MappedCell<'a, String> where Self::Value: ToString {
        return self.map(ToString::to_string)
    }

    fn binded_text<F: 'a + FnMut(&Self::Value) -> S, S: AsRef<str>> (&self, mut f: F) -> Result<web_sys::Text> {
        let text = web_sys::Text::new_with_data(f(self.get().deref()).as_ref())?;
        let my_text = text.clone();
        self.on_update(move |x| my_text.set_data(f(x).as_ref()));
        return Ok(text)
    }
}

pub trait MutableCell<'a>: CellLike<'a> {
    fn mutate<F: FnOnce(&mut Self::Value)> (&mut self, f: F);

    #[inline]
    fn set (&mut self, v: Self::Value) where Self::Value: Sized {
        self.mutate(|x| *x = v)
    }

    #[inline]
    fn update<F: FnOnce(Self::Value) -> Self::Value> (&mut self, f: F) where Self::Value: Sized {
        self.mutate(|x| unsafe {
            let v = core::ptr::read(x);
            core::ptr::write(x, f(v))
        })
    }
}

impl<'a, T: CellLike<'a>> RefComponent for T where T::Value: RefComponent {
    #[inline]
    fn render (&self) -> Result<web_sys::Node> {
        let fragment = DocumentFragment::new()?;

        let s = self.get();
        let mut prev = s.render()?;
        fragment.append_child(&prev)?;

        let my_fragment = fragment.clone();
        self.on_update(move |x| match x.render() {
            Ok(x) => {
                match my_fragment.replace_child(&prev, &x) {
                    Ok(_) => prev = x,
                    Err(e) => wasm_bindgen::throw_val(e)
                }
            },
            Err(e) => wasm_bindgen::throw_val(e)
        });

        return Ok(fragment.into())
    }
}

impl<'a, T: CellLike<'a>> RefAttribute for T where T::Value: RefAttribute {
    #[inline]
    fn render (&self, attr: &web_sys::Attr) -> Result<()> {
        let s = self.get();
        s.render(attr)?;

        let attr = attr.clone();
        self.on_update(move |x| match x.render(&attr) {
            Ok(_) => {},
            Err(e) => wasm_bindgen::throw_val(e)
        });

        Ok(())
    }
}

enum Listener<'a, T: 'a + ?Sized> {
    Once (Box<dyn 'a + FnOnce(&T)>),
    Mut (Box<dyn 'a + FnMut(&T)>)
}

pub struct Cell<'a, T: 'a + ?Sized> {
    listeners: UnsafeCell<Vec<Listener<'a, T>>>,
    v: T
}

impl<'a, T: 'a> Cell<'a, T> {
    #[inline]
    pub const fn new (v: T) -> Self {
        Self { v, listeners: UnsafeCell::new(Vec::new()) }
    }
}

impl<'a, T: 'a + ?Sized> MutableCell<'a> for Cell<'a, T> {
    fn mutate<F: FnOnce(&mut T)> (&mut self, f: F) {
        f(&mut self.v);
        let listeners = self.listeners.get_mut();

        // Wake once
        for f in listeners.drain_filter(|x| matches!(x, Listener::Once(_))) {
            let f = match f {
                Listener::Once(f) => f,
                _ => unsafe { unreachable_unchecked() }
            };
            f(&self.v)
        }

        // Wake mut
        for f in listeners.iter_mut() {
            let f = match f {
                Listener::Mut(f) => f,
                #[cfg(debug_assertions)]
                _ => {
                    jsprintln!("Hello world!");
                    unreachable!()
                },
                #[cfg(not(debug_assertions))]
                _ => unsafe { unreachable_unchecked() }
            };

            f(&self.v)
        }
    }
}

impl<'a, T: 'a + ?Sized> CellLike<'a> for Cell<'a, T> {
    type Value = T;
    type Ref<'b> = &'b T where 'a: 'b;

    #[inline]
    fn get (&self) -> Self::Ref<'_> {
        return &self.v
    }

    #[inline]
    fn on_update<F: 'a + FnMut(&T)> (&self, f: F) {
        let listeners = unsafe { &mut *self.listeners.get() };
        listeners.push(Listener::Mut(Box::new(f)))
    }

    #[inline]
    fn on_update_once<F: 'a + FnOnce(&T)> (&self, f: F) {
        let listeners = unsafe { &mut *self.listeners.get() };
        listeners.push(Listener::Once(Box::new(f)))
    }
}

#[derive(Clone)]
pub struct SharedCell<'a, T: ?Sized> {
    v: Rc<RefCell<Cell<'a, T>>>
}

impl<'a, T: 'a + ?Sized> SharedCell<'a, T> {
    #[inline]
    pub fn new (v: T) -> Self where T: Sized {
        Self { v: Rc::new(RefCell::new(Cell::new(v))) }
    }

    #[inline]
    pub fn set (&self, v: T) where T: Sized {
        self.mutate(|x| *x = v)
    }

    #[inline]
    pub fn update<F: FnOnce(T) -> T> (&self, f: F) where T: Sized {
        self.mutate(|x| unsafe {
            let v = core::ptr::read(x);
            core::ptr::write(x, f(v))
        })
    }

    #[inline]
    pub fn mutate<F: FnOnce(&mut T)> (&self, f: F) {
        let mut cell = self.v.borrow_mut();
        cell.mutate(f);
    }
}

impl<'a, T: 'a + ?Sized> CellLike<'a> for SharedCell<'a, T> {
    type Value = T;
    type Ref<'b> = Ref<'b, Self::Value> where 'a: 'b;

    #[inline]
    fn get (&self) -> Self::Ref<'_> {
        return Ref::map(self.v.borrow(), |x| x.get())
    }

    #[inline]
    fn on_update<F: 'a + FnMut(&Self::Value)> (&self, f: F) {
        self.v.borrow().on_update(f)
    }

    #[inline]
    fn on_update_once<F: 'a + FnOnce(&Self::Value)> (&self, f: F) {
        self.v.borrow().on_update_once(f)
    }
}

impl<'a, T: 'a + ?Sized> MutableCell<'a> for SharedCell<'a, T> {
    #[inline]
    fn mutate<F: FnOnce(&mut Self::Value)> (&mut self, f: F) {
        match Rc::get_mut(&mut self.v) {
            Some(x) => x.get_mut().mutate(f),
            None => self.v.borrow_mut().mutate(f)
        }
    }
}

#[derive(Clone)]
pub struct MappedCell<'a, T> {
    v: Rc<UnsafeCell<Cell<'a, T>>>
}

impl<'a, T: 'a> MappedCell<'a, T> {
    pub fn new<I: ?Sized, C: ?Sized + CellLike<'a, Value = I>, F: 'a + FnMut(&I) -> T> (parent: &C, mut f: F) -> Self {
        let cell = Rc::new(
            UnsafeCell::new(
                Cell::new(
                    f(parent.get().deref())
                )
            )
        );

        let my_cell = cell.clone();
        parent.on_update(move |x| unsafe {
            let v = f(x);
            (&mut *my_cell.get()).set(v)
        });

        return Self { v: cell }
    }
}

impl<'a, T: 'a> CellLike<'a> for MappedCell<'a, T> {
    type Value = T;
    type Ref<'b> = &'b T where 'a: 'b;

    #[inline]
    fn get (&self) -> Self::Ref<'_> {
        let v = unsafe { &*self.v.get() };
        return v.get()
    }

    #[inline]
    fn on_update<F: 'a + FnMut(&T)> (&self, f: F) {
        let v = unsafe { &*self.v.get() };
        v.on_update(f)
    }

    #[inline]
    fn on_update_once<F: 'a + FnOnce(&T)> (&self, f: F) {
        let v = unsafe { &*self.v.get() };
        v.on_update_once(f)
    }
}

/*
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
*/