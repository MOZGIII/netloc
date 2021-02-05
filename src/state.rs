//! [`State`] and associated types.

/// An container that encapsulates the state update logic.
#[derive(Debug, Default)]
pub struct State<T>(Option<T>);

/// The resulting effect of a state update.
#[derive(Debug, PartialEq)]
pub enum UpdateEffect<T> {
    /// The state was initialized with a passed value, and it was empty before the update.
    Initialized,
    /// The state was replaced with a passed value, and it had this value before the update.
    Replaced(T),
    /// The state was unchanged, since the new value is equal to the old value.
    Unchanged,
}

impl<T: PartialEq> State<T> {
    /// Create an uninitialized [`State`].
    pub fn uninitialized() -> Self {
        Self(None)
    }

    /// Update the [`State`] with the new value.
    pub fn update(&mut self, new: T) -> UpdateEffect<T> {
        if let Some(old) = &self.0 {
            if &new == old {
                return UpdateEffect::Unchanged;
            }
        }
        let old = self.0.replace(new);
        match old {
            None => UpdateEffect::Initialized,
            Some(old) => UpdateEffect::Replaced(old),
        }
    }
}
