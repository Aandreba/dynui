use std::{ops::Deref};
use js_sys::Function;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{EventTarget, AddEventListenerOptions};
use crate::{Result, dynui::Attribute, CONTEXT};

#[derive(Debug)]
#[repr(transparent)]
pub struct Node (pub(crate) web_sys::Node);

impl Node {
    #[inline]
    pub unsafe fn new<T: Into<web_sys::Node>> (v: T) -> Self {
        Self(v.into())
    }

    #[inline]
    pub fn append_child<N: Into<Node>> (&self, child: N) -> Result<Node> {
        let child = child.into();
        let v = self.0.append_child(&child.0)?;
        return unsafe { Ok(Self::new(v)) }
    }

    #[inline]
    pub fn remove_child<N: Into<Node>> (&self, child: N) -> Result<Node> {
        let child = child.into();
        let v = self.0.remove_child(&child.0)?;
        return unsafe { Ok(Self::new(v)) }
    }

    #[inline]
    pub fn replace_child<P: Into<Node>, N: Into<Node>> (&self, prev: P, new: N) -> Result<Node> {
        let prev = prev.into();
        let new = new.into();
        let v = self.0.replace_child(&new.0, &prev.0)?;
        return unsafe { Ok(Self::new(v)) }
    }

    // Currently leaks
    #[inline]
    pub fn add_listener<F: 'static + FnMut(web_sys::Event)> (&self, event: &str, f: F) -> Result<()> {
        let listener = Closure::new(f).into_js_value().unchecked_into::<Function>();
        return self.0.add_event_listener_with_callback(event, &listener);
    }

    #[inline]
    pub fn add_once_listener<F: 'static + FnOnce(web_sys::Event)> (&self, event: &str, f: F) -> Result<()> {
        let mut options = AddEventListenerOptions::new();
        let listener = Closure::once_into_js(f).unchecked_into::<Function>();
        return self.0.add_event_listener_with_callback_and_add_event_listener_options(event, &listener, options.once(true));
    }
}

impl Deref for Node {
    type Target = EventTarget;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Element (pub(crate) web_sys::Element);

impl Element {
    #[inline]
    pub unsafe fn new<T: Into<web_sys::Element>> (v: T) -> Self {
        Self(v.into())
    }

    #[inline]
    pub fn set_attribute<T: Attribute> (&self, name: &str, value: T) -> Result<Option<web_sys::Attr>> {
        let attr: web_sys::Attr = CONTEXT.with(|ctx| ctx.document.create_attribute(name))?;
        value.render(&attr)?;
        return self.0.set_attribute_node(&attr)
    }
}

impl Into<Node> for Element {
    #[inline]
    fn into(self) -> Node {
        Node(self.0.into())
    }
}

impl Deref for Element {
    type Target = Node;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe {
            &*(&self.0 as &web_sys::Node as *const web_sys::Node as *const Node)
        }
    }
}

/// A component that can be rendered by mutable reference.
/// A `MutComponent` must be able to be appended to a parent multiple times
pub trait MutComponent {
    fn render (&mut self) -> Result<Node>;
}

impl<T: ?Sized + MutComponent> Component for &mut T {
    #[inline]
    default fn render (self) -> Result<Node> {
        MutComponent::render(self)
    }
}

/// A component that can be rendered by reference.
/// A `RefComponent` must be able to be appended to a parent multiple times
pub trait RefComponent: MutComponent {
    fn render (&self) -> Result<Node>;
}

impl<T: ?Sized + RefComponent> MutComponent for T {
    #[inline]
    default fn render (&mut self) -> Result<Node> {
        RefComponent::render(self)
    }
}

impl<T: ?Sized + RefComponent> Component for &T {
    #[inline]
    default fn render (self) -> Result<Node> {
        RefComponent::render(self)
    }
}

impl RefComponent for web_sys::Text {
    #[inline(always)]
    fn render (&self) -> Result<Node> {
        unsafe {
            return Ok(Node::new(self.clone()))
        }
    }
}

impl RefComponent for &str {
    #[inline]
    fn render (&self) -> Result<Node> {
        RefComponent::render(&web_sys::Text::new_with_data(self)?)
    }
}

impl RefComponent for str {
    #[inline]
    fn render (&self) -> Result<Node> {
        RefComponent::render(&web_sys::Text::new_with_data(self)?)
    }
}

impl RefComponent for String {
    #[inline]
    fn render (&self) -> Result<Node> {
        RefComponent::render(&web_sys::Text::new_with_data(self)?)
    }
}

impl RefComponent for Box<str> {
    #[inline]
    fn render (&self) -> Result<Node> {
        RefComponent::render(&web_sys::Text::new_with_data(self)?)
    }
}

/// A component that can be rendered by ownership.
/// A `Component` may only be appended to a parent once.
pub trait Component {
    fn render (self) -> Result<Node>;
}

impl<T: Component> Component for Result<T> {
    #[inline]
    fn render (self) -> Result<Node> {
        self.and_then(T::render)
    }
}

impl Component for Node {
    #[inline(always)]
    fn render (self) -> Result<Node> {
        Ok(self)
    }
}

impl Component for Element {
    #[inline(always)]
    fn render (self) -> Result<Node> {
        Ok(self.into())
    }
}

impl Component for web_sys::Text {
    #[inline(always)]
    fn render (self) -> Result<Node> {
        unsafe {
            return Ok(Node::new(self))
        }
    }
}