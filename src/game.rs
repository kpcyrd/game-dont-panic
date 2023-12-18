pub const START_Y: u8 = 18;
const MAX_Y: u8 = 34;
const STEP_Y: u8 = 2;

pub enum Direction {
    Clockwise,
    CounterClock,
}

#[derive(PartialEq)]
pub enum Gun {
    Revolver,
    Scorpio,
}

pub enum Action {
    Rotate(Direction),
    ReloadToggle,
    Shoot,
}

#[derive(Clone, Copy)]
pub enum Screen {
    Start,
    Normal,
    Reload,
}

pub struct Game {
    screen: Screen,
    gun: Gun,
    score: u32,
    y: u8,
}

impl Default for Game {
    fn default() -> Self {
        Game {
            screen: Screen::Start,
            gun: Gun::Revolver,
            score: 0,
            y: START_Y,
        }
    }
}

impl Game {
    pub fn new() -> Game {
        Game::default()
    }

    pub fn screen(&self) -> Screen {
        self.screen
    }

    pub fn y(&self) -> u8 {
        self.y
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn action(&mut self, action: &Action) {
        match (self.screen, action) {
            // start screen
            (Screen::Start, Action::ReloadToggle) => {
                *self = Game {
                    screen: Screen::Normal,
                    score: 1330,
                    ..Default::default()
                };
            }
            (Screen::Start, _) => {}
            // default screen
            (Screen::Normal, Action::Rotate(Direction::Clockwise)) => {
                if self.y + STEP_Y <= MAX_Y {
                    self.y += STEP_Y;
                }
            }
            (Screen::Normal, Action::Rotate(Direction::CounterClock)) => {
                self.y = self.y.saturating_sub(STEP_Y);
            }
            (Screen::Normal, Action::ReloadToggle) => {
                // only the revolver can be reloaded
                if self.gun == Gun::Revolver {
                    self.screen = Screen::Reload;
                }
            }
            (Screen::Normal, Action::Shoot) => {
                self.score = self.score.saturating_add(1);
            }
            // reload screen
            (Screen::Reload, Action::Rotate(Direction::Clockwise)) => {}
            (Screen::Reload, Action::Rotate(Direction::CounterClock)) => {}
            (Screen::Reload, Action::ReloadToggle) => {
                self.screen = Screen::Normal;
            }
            // misc
            (_, Action::Shoot) => {
                *self = Game {
                    screen: Screen::Start,
                    ..Default::default()
                };
            }
        }
    }
}
