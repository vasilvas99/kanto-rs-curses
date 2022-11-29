pub mod kanto_api;
pub mod containers_table_view;

pub fn try_best<T>(err: T) {
    // Used to consume Err variants where they can be safely ignored.
    // Using it means that we try an operation to the best of our abilities
    // but failures can be (safely) ignored. E.g. we try to send a request down a
    // channel but if it's full we don't do anything
    std::mem::drop(err);
}