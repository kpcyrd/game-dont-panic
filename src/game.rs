use fugit::Duration;

pub const START_Y: u8 = 18;
const MAX_Y: u8 = 34;
const STEP_Y: u8 = 2;

// this is not 100ms, I don't know how to configure this properly
pub const TICK_INTERVAL: Duration<u64, 1, 8> = Duration::<u64, 1, 8>::millis(100);
pub const DEBOUNCE_TICKS: u8 = 1;
pub const ROUND_TICKS: u8 = 10;

pub enum Direction {
    Clockwise,
    CounterClock,
}

#[derive(PartialEq)]
pub enum Gun {
    Revolver,
    // Scorpio,
}

pub enum Button {
    ReloadToggle,
    Shoot,
}

pub enum Action {
    Rotate(Direction),
    Press(Button),
    Release(Button),
}

#[derive(Clone, Copy, PartialEq)]
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

    reload_toggle_debounce: u8,
    shoot_debounce: u8,
    round_ticks: u8,
}

impl Default for Game {
    fn default() -> Self {
        Game {
            screen: Screen::Start,
            gun: Gun::Revolver,
            score: 0,
            y: START_Y,

            reload_toggle_debounce: 0,
            shoot_debounce: 0,
            round_ticks: 0,
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

    pub fn add_score(&mut self, points: u32) {
        self.score = self.score.saturating_add(points);
    }

    pub fn tick(&mut self) {
        if self.reload_toggle_debounce > 0 {
            self.reload_toggle_debounce -= 1;
        }

        if self.shoot_debounce > 0 {
            self.shoot_debounce -= 1;
        }

        if self.screen != Screen::Start {
            if self.round_ticks > 0 {
                self.round_ticks -= 1;
            } else {
                self.round_ticks = ROUND_TICKS;

                // TODO: the game executes one tick
                // self.add_score(1);
            }
        }
    }

    fn debounce_for_button(&mut self, button: &Button) -> &mut u8 {
        match button {
            Button::ReloadToggle => &mut self.reload_toggle_debounce,
            Button::Shoot => &mut self.shoot_debounce,
        }
    }

    pub fn action(&mut self, action: &Action) {
        // debouncing
        match action {
            Action::Press(button) => {
                let debounce = self.debounce_for_button(button);
                if *debounce > 0 {
                    return;
                }
            }
            Action::Release(button) => {
                let debounce = self.debounce_for_button(button);
                *debounce = DEBOUNCE_TICKS;
            }
            _ => (),
        }

        // process action
        match (self.screen, action) {
            // start screen
            (Screen::Start, Action::Press(Button::Shoot)) => {
                *self = Game {
                    screen: Screen::Normal,
                    score: 1337,
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
            (Screen::Normal, Action::Press(Button::ReloadToggle)) => {
                // only the revolver can be reloaded
                if self.gun == Gun::Revolver {
                    self.screen = Screen::Reload;
                }
            }
            (Screen::Normal, Action::Press(Button::Shoot)) => {
                self.add_score(1);
            }
            // reload screen
            (Screen::Reload, Action::Rotate(Direction::Clockwise)) => {}
            (Screen::Reload, Action::Rotate(Direction::CounterClock)) => {}
            (Screen::Reload, Action::Press(Button::ReloadToggle)) => {
                self.screen = Screen::Normal;
            }
            // misc
            (_, Action::Press(Button::Shoot)) => {
                *self = Game {
                    screen: Screen::Start,
                    ..Default::default()
                };
            }
            (_, Action::Release(_)) => (),
        }
    }
}
