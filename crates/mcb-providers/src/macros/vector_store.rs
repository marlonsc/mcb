//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md#vector-store-providers)
//!
//! Vector store provider macros.
//!
//! - `send_actor_msg!`: Actor message send/await pattern (edgevec only)

/// Send a message to the `EdgeVec` actor and await the oneshot response.
///
/// Constructs the message with a oneshot channel, sends it via the actor's
/// sender, and awaits the response.
///
/// ## Example
///
/// ```ignore
/// send_actor_msg!(self, Core(CoreMessage::CreateCollection {
///     name: collection.to_string()
/// }))
/// ```
macro_rules! send_actor_msg {
    (
        $self:expr,
        $outer:ident ( $inner:ident :: $variant:ident {
            $($field:ident : $val:expr),* $(,)?
        })
    ) => {{
        let (tx, rx) = tokio::sync::oneshot::channel();
        let _ = $self
            .sender
            .send(EdgeVecMessage::$outer($inner::$variant {
                $($field: $val,)*
                tx,
            }))
            .await;
        rx.await
            .unwrap_or_else(|_| Err(mcb_domain::error::Error::internal("Actor closed")))
    }};
}
