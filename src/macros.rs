#[macro_export]
/// Locks an `Arc<Mutex<T>>` and returns `<T>::clone()`.
macro_rules! lock_and_clone {
    ($arc_mutex:expr, $value_to_clone:ident) => {
        $arc_mutex.lock().unwrap().$value_to_clone.clone()
    };
    ($arc_mutex:expr, $inner_arc_mutex:ident, $value_to_clone:ident) => {
        lock_and_clone!(lock_and_clone!($arc_mutex, $inner_arc_mutex), $value_to_clone)
    };
}
