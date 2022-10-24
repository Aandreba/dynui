use crate::Result;

pub trait RefComponent: Component {
    fn render (&self) -> Result<&web_sys::Node>;
}

impl<T: RefComponent> Component for &T {
    #[inline]
    fn render (self) -> Result<web_sys::Node> {
        RefComponent::render(self).map(Clone::clone)
    }
}

impl RefComponent for web_sys::Node {
    #[inline(always)]
    fn render (&self) -> Result<&web_sys::Node> {
        Ok(self)
    }
}

impl RefComponent for web_sys::Element {
    #[inline(always)]
    fn render (&self) -> Result<&web_sys::Node> {
        Ok(self)
    }
}

impl RefComponent for web_sys::Text {
    #[inline(always)]
    fn render (&self) -> Result<&web_sys::Node> {
        Ok(self)
    }
}

impl RefComponent for web_sys::DocumentFragment {
    #[inline(always)]
    fn render (&self) -> Result<&web_sys::Node> {
        Ok(self)
    }
}

/// Component
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

impl Component for &str {
    #[inline]
    fn render (self) -> Result<web_sys::Node> {
        web_sys::Text::new_with_data(self).map(Into::into)
    }
}

impl Component for String {
    #[inline]
    fn render (self) -> Result<web_sys::Node> {
        web_sys::Text::new_with_data(&self).map(Into::into)
    }
}

impl Component for Box<str> {
    #[inline]
    fn render (self) -> Result<web_sys::Node> {
        web_sys::Text::new_with_data(&self).map(Into::into)
    }
}