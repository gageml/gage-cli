pub mod confirm;
pub mod help;
pub mod notify;
pub mod status;

#[macro_export]
macro_rules! handle_wrapped_dialog_event {
    ($self:ident, $event:ident) => {
        $self
            .with_view_mut(|v| v.on_event($event))
            .map(|e| match e {
                EventResult::Ignored => EventResult::consumed(),
                _ => e,
            })
            .unwrap_or(EventResult::Ignored)
    };
}
