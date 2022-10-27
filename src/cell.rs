use std::{hint::unreachable_unchecked, ops::Deref, rc::Rc, cell::{UnsafeCell, RefCell, Ref}, fmt::Debug};
use crate::{Result, component::{RefComponent, Node}, attr::RefAttribute, jseprintln};

/// An object that can be treated like a cell.
/// Cells are objects that notify other parts of the code when their underlying value is mutated.
pub trait CellLike {
    type Value: ?Sized;
    type Ref<'b>: 'b + Deref<Target = Self::Value> where Self: 'b;
    
    /// Returns a reference to the current value of the cell
    fn get (&self) -> Self::Ref<'_>;
    /// Sets up a callback to be executed whenever the cell's state changes
    fn on_update<F: 'static + FnMut(&Self::Value)> (&self, f: F);
    /// Sets up a callback to be executed whenever the next mutation to the state ocurrs
    fn on_update_once<F: 'static + FnOnce(&Self::Value)> (&self, f: F);

    #[inline]
    fn zipped_map<T: 'static, C: 'static + ?Sized + CellLike + Clone, F: 'static + FnMut(&Self::Value, &C::Value) -> T> (&self, other: &C, f: F) -> ZippedCell<T> where Self: 'static + Clone {
        return ZippedCell::new(self, other, f)
    }
    
    /// Maps the [`CellLike`]'s value with the specified `f`. 
    /// Every time the original cell mutates, so will the mapped cell
    #[inline]
    fn map<T: 'static, F: 'static + FnMut(&Self::Value) -> T> (&self, f: F) -> MappedCell<T> {
        return MappedCell::new::<Self, F>(self, f)
    }

    /// Maps the [`CellLike`] to its value's [`Debug`] representation
    #[inline]
    fn debug (&self) -> MappedCell<String> where Self::Value: 'static + Debug {
        return self.map(|x| format!("{x:?}"))
    }
    
    /// Maps the [`CellLike`] to its value's [`Display`](std::fmt::Display) representation
    #[inline]
    fn display (&self) -> MappedCell<String> where Self::Value: 'static + ToString {
        return self.map(ToString::to_string)
    }
}

/// A cell who's state can be mutated.
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

/// A cell that can be mutated through a shared reference.
pub trait RefMutableCell: MutableCell {
    fn mutate<F: FnOnce(&mut Self::Value)> (&self, f: F);

    #[inline]
    fn set (&self, v: Self::Value) where Self::Value: Sized {
        self.mutate(|x| *x = v)
    }

    #[inline]
    fn update<F: FnOnce(Self::Value) -> Self::Value> (&self, f: F) where Self::Value: Sized {
        self.mutate(|x| unsafe {
            let v = core::ptr::read(x);
            core::ptr::write(x, f(v))
        })
    }
}

impl<T: RefMutableCell> MutableCell for T {
    #[inline]
    default fn mutate<F: FnOnce(&mut Self::Value)> (&mut self, f: F) {
        RefMutableCell::mutate(self, f)
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

/// Basic cell
#[derive(Default)]
pub struct Cell<T: ?Sized> {
    listeners: UnsafeCell<Vec<Listener<T>>>,
    v: T
}

impl<T> Cell<T> {
    /// Creates a new [`Cell`] with the specified
    #[inline]
    pub const fn new (v: T) -> Self {
        Self { v, listeners: UnsafeCell::new(Vec::new()) }
    }

    #[inline]
    pub fn into_shared (self) -> SharedCell<T> {
        return SharedCell { v: Rc::new(RefCell::new(self)) }
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

#[derive(Default)]
pub struct SharedCell<T: ?Sized> {
    v: Rc<RefCell<Cell<T>>>
}

impl<T: ?Sized> SharedCell<T> {
    #[inline]
    pub fn new (v: T) -> Self where T: Sized {
        Self { v: Rc::new(RefCell::new(Cell::new(v))) }
    }
}

impl<T: ?Sized> RefMutableCell for SharedCell<T> {
    #[inline]
    fn mutate<F: FnOnce(&mut Self::Value)> (&self, f: F) {
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

impl<T: ?Sized> Clone for SharedCell<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self { v: self.v.clone() }
    }
}

#[repr(transparent)]
pub struct MappedCell<T: ?Sized> {
    v: SharedCell<T>
}

impl<T: 'static> MappedCell<T> {
    #[inline]
    pub fn new<C: ?Sized + CellLike, F: 'static + FnMut(&C::Value) -> T> (parent: &C, mut f: F) -> Self {
        let cell = SharedCell::new(f(parent.get().deref()));
        let my_cell = cell.clone();
        parent.on_update(move |x| RefMutableCell::set(&my_cell, f(x)));
        return Self { v: cell }
    }
}

impl<T: 'static> CellLike for MappedCell<T> {
    type Value = T;
    type Ref<'b> = <SharedCell<T> as CellLike>::Ref<'b>;

    #[inline]
    fn get (&self) -> Self::Ref<'_> {
        return self.v.get();
    }

    #[inline]
    fn on_update<F: 'static + FnMut(&T)> (&self, f: F) {
        self.v.on_update(f)
    }

    #[inline]
    fn on_update_once<F: 'static + FnOnce(&T)> (&self, f: F) {
        self.v.on_update_once(f)
    }
}

impl<T: ?Sized> Clone for MappedCell<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self { v: self.v.clone() }
    }
}

#[repr(transparent)]
pub struct ZippedCell<T: ?Sized> {
    v: SharedCell<T>
}

impl<T: 'static> ZippedCell<T> {
    pub fn new<L: 'static + ?Sized + CellLike + Clone, R: 'static + ?Sized + CellLike + Clone, F: 'static + FnMut(&L::Value, &R::Value) -> T> (lhs: &L, rhs: &R, mut f: F) -> Self {
        let cell = SharedCell::new(f(lhs.get().deref(), rhs.get().deref()));
        let f = Rc::new(UnsafeCell::new(f)) as Rc<UnsafeCell<dyn FnMut(&L::Value, &R::Value) -> T>>;

        let my_cell = cell.clone();
        let my_rhs = rhs.clone();
        let my_f = f.clone();
        lhs.on_update(move |x| unsafe {
            let f = &mut *UnsafeCell::get(&my_f);
            RefMutableCell::set(&my_cell, f(x, my_rhs.get().deref()))
        });

        let my_cell = cell.clone();
        let my_lhs = lhs.clone();
        rhs.on_update(move |x| unsafe {
            let f = &mut *UnsafeCell::get(&f);
            RefMutableCell::set(&my_cell, f(my_lhs.get().deref(), x))
        });
        
        return Self { v: cell }
    }
}

impl<T: 'static> CellLike for ZippedCell<T> {
    type Value = T;
    type Ref<'b> = <SharedCell<T> as CellLike>::Ref<'b>;

    #[inline]
    fn get (&self) -> Self::Ref<'_> {
        return self.v.get();
    }

    #[inline]
    fn on_update<F: 'static + FnMut(&T)> (&self, f: F) {
        self.v.on_update(f)
    }

    #[inline]
    fn on_update_once<F: 'static + FnOnce(&T)> (&self, f: F) {
        self.v.on_update_once(f)
    }
}

impl<T: ?Sized> Clone for ZippedCell<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self { v: self.v.clone() }
    }
}