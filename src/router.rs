pub struct Router<T> {
    pub route: T,
}

impl<T> Router<T> {
    pub fn new(default_route: T) -> Self {
        Self {
            route: default_route,
        }
    }
}
