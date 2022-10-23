pub trait Component {
    type Node;

    fn render (self) -> Self::Node;
}