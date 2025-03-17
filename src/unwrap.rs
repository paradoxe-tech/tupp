pub trait UnwrapString<T> {
    fn unwrap_string(&self) -> T;
}

impl UnwrapString<String> for Option<String> {
    fn unwrap_string(&self) -> String {
        self.as_deref().unwrap_or("").to_string()
    }
}