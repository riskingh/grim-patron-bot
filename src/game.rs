pub struct Game {
    started: bool,
}

impl Game {
    pub fn new() -> Game {
        Game{started: false}
    }

    pub fn start(&mut self) {
        assert!(!self.started);
        self.started = true;
    }

    pub fn is_started(&self) -> bool {
        self.started
    }
}
