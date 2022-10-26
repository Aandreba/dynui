use std::{hint::unreachable_unchecked, ops::Deref, rc::Rc, cell::{UnsafeCell, RefCell, Ref}, fmt::Debug};
use crate::{Result, component::{RefComponent, Node}, attr::RefAttribute, jseprintln};

/// An object that can be treated like a cell
pub trait CellLike {
    type Value: ?Sized;
    type Ref<'b>: 'b + Deref<Target = Self::Value> where Self: 'b;
    
    fn get (&self) -> Self::Ref<'_>;
    fn on_update<F: 'static + FnMut(&Self::Value)> (&self, f: F);
    fn on_update_once<F: 'static + FnOnce(&Self::Value)> (&self, f: F);

    #[inline]
    fn map<T: 'static, F: 'static + FnMut(&Self::Value) -> T> (&self, f: F) -> MappedCell<T> {
        return MappedCell::new::<Self::Value, Self, F>(self, f)
    }

    #[inline]
    fn debug (&self) -> MappedCell<String> where Self::Value: 'static + Debug {
        return self.map(|x| format!("{x:?}"))
    }
    
    #[inline]
    fn display (&self) -> MappedCell<String> where Self::Value: 'static + ToString {
        return self.map(ToString::to_string)
    }

    fn binded_text<F: 'static + FnMut(&Self::Value) -> S, S: AsRef<str>> (&self, mut f: F) -> Result<web_sys::Text> {
        let text = web_sys::Text::new_with_data(f(self.get().deref()).as_ref())?;
        let my_text = text.clone();
        self.on_update(move |x| my_text.set_data(f(x).as_ref()));
        return Ok(text)
    }
}

pub trait MutableCell: CellLike {
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

impl<T: CellLike> RefComponent for T where <T as CellLike>::Value: RefComponent {
    fn render (&self) -> Result<Node> {
        let s = self.get();
        let prev = RefComponent::render(s.deref())?;
        let mut my_prev = prev.0.clone();

        self.on_update(move |x| match RefComponent::render(x) {
            Ok(x) => match my_prev.parent_node() {
                Some(parent) => match parent.replace_child(&x.0, &my_prev) {
                    Ok(_) => my_prev = x.0,
                    Err(e) => wasm_bindgen::throw_val(e)
                },
                None => {
                    #[cfg(debug_assertions)]
                    jseprintln!("previous node doesn't have a parent")
                }
            },
            Err(e) => wasm_bindgen::throw_val(e)
        });

        return Ok(prev)
    }
}

impl<T: CellLike> RefAttribute for T where T::Value: RefAttribute {
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

enum Listener<T: ?Sized> {
    Once (Box<dyn 'static + FnOnce(&T)>),
    Mut (Box<dyn 'static + FnMut(&T)>)
}

pub struct Cell<T: ?Sized> {
    listeners: UnsafeCell<Vec<Listener<T>>>,
    v: T
}

impl<T> Cell<T> {
    #[inline]
    pub const fn new (v: T) -> Self {
        Self { v, listeners: UnsafeCell::new(Vec::new()) }
    }
}

impl<T: ?Sized> MutableCell for Cell<T> {
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
                    unreachable!()
                },
                #[cfg(not(debug_assertions))]
                _ => unsafe { unreachable_unchecked() }
            };

            f(&self.v)
        }
    }
}

impl<T: ?Sized> CellLike for Cell<T> {
    type Value = T;
    type Ref<'b> = &'b T where T: 'b;

    #[inline]
    fn get (&self) -> Self::Ref<'_> {
        return &self.v
    }

    #[inline]
    fn on_update<F: 'static + FnMut(&T)> (&self, f: F) {
        let listeners = unsafe { &mut *self.listeners.get() };
        listeners.push(Listener::Mut(Box::new(f)))
    }

    #[inline]
    fn on_update_once<F: 'static + FnOnce(&T)> (&self, f: F) {
        let listeners = unsafe { &mut *self.listeners.get() };
        listeners.push(Listener::Once(Box::new(f)))
    }
}

#[derive(Clone)]
pub struct SharedCell<T: ?Sized> {
    v: Rc<RefCell<Cell<T>>>
}

impl<T: ?Sized> SharedCell<T> {
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

impl<T: ?Sized> CellLike for SharedCell<T> {
    type Value = T;
    type Ref<'b> = Ref<'b, Self::Value> where Self::Value: 'b;

    #[inline]
    fn get (&self) -> Self::Ref<'_> {
        return Ref::map(self.v.borrow(), |x| x.get())
    }

    #[inline]
    fn on_update<F: 'static + FnMut(&Self::Value)> (&self, f: F) {
        self.v.borrow().on_update(f)
    }

    #[inline]
    fn on_update_once<F: 'static + FnOnce(&Self::Value)> (&self, f: F) {
        self.v.borrow().on_update_once(f)
    }
}

impl<T: ?Sized> MutableCell for SharedCell<T> {
    #[inline]
    fn mutate<F: FnOnce(&mut Self::Value)> (&mut self, f: F) {
        match Rc::get_mut(&mut self.v) {
            Some(x) => x.get_mut().mutate(f),
            None => self.v.borrow_mut().mutate(f)
        }
    }
}

#[derive(Clone)]
pub struct MappedCell<T> {
    v: Rc<UnsafeCell<Cell<T>>>
}

impl<T: 'static> MappedCell<T> {
    pub fn new<I: ?Sized, C: ?Sized + CellLike<Value = I>, F: 'static + FnMut(&I) -> T> (parent: &C, mut f: F) -> Self {
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

impl<T> CellLike for MappedCell<T> {
    type Value = T;
    type Ref<'b> = &'b T where T: 'b;

    #[inline]
    fn get (&self) -> Self::Ref<'_> {
        let v = unsafe { &*self.v.get() };
        return v.get()
    }

    #[inline]
    fn on_update<F: 'static + FnMut(&T)> (&self, f: F) {
        let v = unsafe { &*self.v.get() };
        v.on_update(f)
    }

    #[inline]
    fn on_update_once<F: 'static + FnOnce(&T)> (&self, f: F) {
        let v = unsafe { &*self.v.get() };
        v.on_update_once(f)
    }
}