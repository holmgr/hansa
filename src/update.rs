use ggez::Context;

/// An updatable type, which internal data needs to be updated based on
/// game time etc.
pub trait Updatable<'a> {
    /// Environmental data needed to update.
    type Data;

    /// Updates the internal data of the type.
    fn update(&'a mut self, ctx: &'a Context, data: Self::Data);
}
