use crate::engine::{field_i64, ClientEvent};

#[allow(dead_code)]
pub(super) fn same_tick_count(
    events: &[ClientEvent],
    name: &str,
    key: &str,
    value: i64,
    min_count: usize,
) -> bool {
    events.iter().any(|anchor| {
        anchor.name == name
            && field_i64(anchor, key) == Some(value)
            && events
                .iter()
                .filter(|event| {
                    event.t == anchor.t
                        && event.name == name
                        && field_i64(event, key) == Some(value)
                })
                .count()
                >= min_count
    })
}

#[allow(dead_code)]
pub(super) fn has(events: &[ClientEvent], name: &str, key: &str, value: i64) -> bool {
    events
        .iter()
        .any(|event| event.name == name && field_i64(event, key) == Some(value))
}
