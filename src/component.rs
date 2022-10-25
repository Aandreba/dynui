use crate::Result;

/// A component that can be rendered by mutable reference
pub trait MutComponent {
    fn render (&mut self) -> Result<web_sys::Node>;
}

impl<T: ?Sized + MutComponent> Component for &mut T {
    #[inline]
    default fn render (self) -> Result<web_sys::Node> {
        MutComponent::render(self)
    }
}

/// A component that can be rendered by reference
pub trait RefComponent: MutComponent {
    fn render (&self) -> Result<web_sys::Node>;
}

impl<T: ?Sized + RefComponent> MutComponent for T {
    #[inline]
    default fn render (&mut self) -> Result<web_sys::Node> {
        RefComponent::render(self)
    }
}

impl<T: ?Sized + RefComponent> Component for &T {
    #[inline]
    default fn render (self) -> Result<web_sys::Node> {
        RefComponent::render(self)
    }
}

impl RefComponent for web_sys::Node {
    #[inline(always)]
    fn render (&self) -> Result<web_sys::Node> {
        Ok(self.clone())
    }
}

impl RefComponent for web_sys::Text {
    #[inline(always)]
    fn render (&self) -> Result<web_sys::Node> {
        Ok(self.clone().into())
    }
}

impl RefComponent for web_sys::HtmlElement {
    #[inline(always)]
    fn render (&self) -> Result<web_sys::Node> {
        Ok(self.clone().into())
    }
}

impl RefComponent for web_sys::Element {
    #[inline(always)]
    fn render (&self) -> Result<web_sys::Node> {
        Ok(self.clone().into())
    }
}

impl RefComponent for web_sys::DocumentFragment {
    #[inline(always)]
    fn render (&self) -> Result<web_sys::Node> {
        Ok(self.clone().into())
    }
}

impl RefComponent for &str {
    #[inline]
    fn render (&self) -> Result<web_sys::Node> {
        web_sys::Text::new_with_data(self).map(Into::into)
    }
}

impl RefComponent for str {
    #[inline]
    fn render (&self) -> Result<web_sys::Node> {
        web_sys::Text::new_with_data(self).map(Into::into)
    }
}

impl RefComponent for String {
    #[inline]
    fn render (&self) -> Result<web_sys::Node> {
        web_sys::Text::new_with_data(self).map(Into::into)
    }
}

impl RefComponent for Box<str> {
    #[inline]
    fn render (&self) -> Result<web_sys::Node> {
        web_sys::Text::new_with_data(self).map(Into::into)
    }
}

/// A component that can be rendered by ownership
pub trait Component {
    fn render (self) -> Result<web_sys::Node>;
}

impl<T: Component> Component for Result<T> {
    #[inline]
    fn render (self) -> Result<web_sys::Node> {
        self.and_then(T::render)
    }
}

impl Component for web_sys::Node {
    #[inline(always)]
    fn render (self) -> Result<web_sys::Node> {
        Ok(self)
    }
}

impl Component for web_sys::Text {
    #[inline(always)]
    fn render (self) -> Result<web_sys::Node> {
        Ok(self.into())
    }
}

impl Component for web_sys::HtmlElement {
    #[inline(always)]
    fn render (self) -> Result<web_sys::Node> {
        Ok(self.into())
    }
}

impl Component for web_sys::Element {
    #[inline(always)]
    fn render (self) -> Result<web_sys::Node> {
        Ok(self.into())
    }
}

impl Component for web_sys::DocumentFragment {
    #[inline(always)]
    fn render (self) -> Result<web_sys::Node> {
        Ok(self.into())
    }
}