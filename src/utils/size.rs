use freya::prelude::*;

/// Methods for setting an element's width and height.
pub trait SizeExt
where
    Self: ContainerSizeExt,
{
    /// Set the element's height and width. See [`Size`].
    fn size(mut self, size: impl Into<Size>) -> Self {
        let size = size.into();
        let layout = self.get_layout();

        layout.layout.width = size.clone();
        layout.layout.height = size;

        self
    }
}

impl<T: ContainerSizeExt> SizeExt for T {}
