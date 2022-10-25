use std::{rc::Rc, sync::Arc};

use crate::Result;

/// Attribute that can be rendered with ownership
pub trait Attribute {
    fn render (self, attr: &web_sys::Attr) -> Result<()>;
}

/// Attribute that can be rendered with a reference
pub trait RefAttribute: MutAttribute {
    fn render (&self, attr: &web_sys::Attr) -> Result<()>;
}

impl<T: ?Sized + RefAttribute> Attribute for &T {
    #[inline]
    default fn render (self, attr: &web_sys::Attr) -> Result<()> {
        RefAttribute::render(self, attr)
    }
}

impl<T: ?Sized + RefAttribute> MutAttribute for T {
    #[inline]
    default fn render (&mut self, attr: &web_sys::Attr) -> Result<()> {
        RefAttribute::render(self, attr)
    }
}

impl RefAttribute for &str {
    #[inline]
    fn render (&self, attr: &web_sys::Attr) -> Result<()> {
        attr.set_value(self);
        Ok(())
    }
}

impl RefAttribute for str {
    #[inline]
    fn render (&self, attr: &web_sys::Attr) -> Result<()> {
        attr.set_value(self);
        Ok(())
    }
}

impl RefAttribute for String {
    #[inline]
    fn render (&self, attr: &web_sys::Attr) -> Result<()> {
        attr.set_value(self);
        Ok(())
    }
}

impl RefAttribute for Box<str> {
    #[inline]
    fn render (&self, attr: &web_sys::Attr) -> Result<()> {
        attr.set_value(self);
        Ok(())
    }
}

impl RefAttribute for Rc<str> {
    #[inline]
    fn render (&self, attr: &web_sys::Attr) -> Result<()> {
        attr.set_value(self);
        Ok(())
    }
}

impl RefAttribute for Arc<str> {
    #[inline]
    fn render (&self, attr: &web_sys::Attr) -> Result<()> {
        attr.set_value(self);
        Ok(())
    }
}

/// Attribute that can be rendered with mutable reference
pub trait MutAttribute {
    fn render (&mut self, attr: &web_sys::Attr) -> Result<()>;
}

impl<T: ?Sized + MutAttribute> Attribute for &mut T {
    #[inline]
    default fn render (self, attr: &web_sys::Attr) -> Result<()> {
        MutAttribute::render(self, attr)
    }
}