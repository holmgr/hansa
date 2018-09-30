use ggez::Context;

/// An updatable type, which internal data needs to be updated based on 
/// game time etc.
pub trait Updatable {
    /// Environmental data needed to update.
    type Data;

    /// Updates the internal data of the type.
    fn update(&mut self, ctx: &Context, data: &Self::Data);
}
