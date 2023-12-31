use crate::gfx;
use crate::guns::{self, Gun, Revolver, Scorpio};
use crate::opps::{self, Lawn};
use fugit::Duration;
use rand_core::RngCore;

pub const START_Y: u8 = 18;
const STEP_Y: u8 = 2;

// this is not 100ms, I don't know how to configure this properly
pub const TICK_INTERVAL: Duration<u64, 1, 8> = Duration::<u64, 1, 8>::millis(100);
pub const DEBOUNCE_TICKS: u8 = 1;

// quick sanity check so we don't spawn unreachable opponents
static_assertions::const_assert!(gfx::FERRIS_MAX_Y + guns::REVOLVER_OFFSET >= opps::MAX_SPAWN_Y);
static_assertions::const_assert!(gfx::FERRIS_MAX_Y + guns::SCORPIO_OFFSET >= opps::MAX_SPAWN_Y);

pub enum Direction {
    Clockwise,
    CounterClock,
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
    Wasted,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Rumble {
    Start((u8, u16)),
    On(u8),
}

pub struct Game {
    screen: Screen,
    score: u32,
    y: u8,

    primary_gun: Option<Scorpio>,
    pub secondary_gun: Revolver,
    next_shot: Option<u8>,
    pub lawn: opps::Lawn,
    reload_toggle_debounce: u8,
    shoot_debounce: u8,

    pub rumble: Option<Rumble>,
}

impl Default for Game {
    fn default() -> Self {
        Game {
            screen: Screen::Start,
            score: 0,
            y: START_Y,

            // primary_gun: Some(Scorpio::new()),
            primary_gun: None,
            secondary_gun: Revolver::new(),
            next_shot: None,
            lawn: Lawn::default(),
            reload_toggle_debounce: 0,
            shoot_debounce: 0,

            rumble: None,
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

    pub fn gun(&mut self) -> Gun<'_> {
        self.primary_gun
            .as_mut()
            .map(Gun::Scorpio)
            .unwrap_or(Gun::Revolver(&mut self.secondary_gun))
    }

    pub fn shoot(&mut self) {
        match self.gun().shoot() {
            // did fire
            Some((true, offset)) => {
                if self.lawn.shoot(self.y + offset) {
                    self.add_score(1);
                }
                self.rumble = Some(Rumble::Start((
                    guns::SHOT_RUMBLE_TICKS,
                    guns::SHOT_RUMBLE_DIVIDER,
                )));
            }
            // did not fire (but gun is not used up)
            Some((false, _)) => (),
            // primary weapon is used up
            None => {
                self.primary_gun = None;
            }
        }
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn add_score(&mut self, points: u32) {
        self.score = self.score.saturating_add(points);
        if self.primary_gun.is_some() {
            return;
        }

        // bonus weapon drop
        if self.score % 10 == 0 {
            self.primary_gun = Some(Scorpio::new());
        }
    }

    pub fn tick<R: RngCore>(&mut self, random: R) {
        if self.reload_toggle_debounce > 0 {
            self.reload_toggle_debounce -= 1;
        }

        if self.shoot_debounce > 0 {
            self.shoot_debounce -= 1;
        }

        if let Some(Rumble::On(duration)) = &mut self.rumble {
            *duration -= 1;
        }

        if let Screen::Normal | Screen::Reload = self.screen {
            if let Some(next_shot) = self.next_shot {
                let next_shot = next_shot.saturating_sub(1);
                if next_shot == 0 {
                    self.shoot();
                    self.schedule_next_shot();
                } else {
                    self.next_shot = Some(next_shot);
                }
            }

            if self.lawn.tick(self.score, random) {
                self.screen = Screen::Wasted;
            }
        }
    }

    fn debounce_for_button(&mut self, button: &Button) -> &mut u8 {
        match button {
            Button::ReloadToggle => &mut self.reload_toggle_debounce,
            Button::Shoot => &mut self.shoot_debounce,
        }
    }

    fn schedule_next_shot(&mut self) {
        match self.gun() {
            Gun::Revolver(_) => (),
            Gun::Scorpio(_) => {
                self.next_shot = Some(Scorpio::FIRE_RATE);
            }
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
                    ..Default::default()
                };
                self.shoot();
            }
            (Screen::Start, _) => {}
            // default screen
            (Screen::Normal, Action::Rotate(Direction::Clockwise)) => {
                if self.y + STEP_Y <= gfx::FERRIS_MAX_Y {
                    self.y += STEP_Y;
                }
            }
            (Screen::Normal, Action::Rotate(Direction::CounterClock)) => {
                self.y = self.y.saturating_sub(STEP_Y);
            }
            (Screen::Normal, Action::Press(Button::ReloadToggle)) => {
                // only the revolver can be reloaded
                match self.gun() {
                    Gun::Revolver(_) => {
                        self.screen = Screen::Reload;
                    }
                    Gun::Scorpio(_) => (),
                }
            }
            (Screen::Normal, Action::Press(Button::Shoot)) => {
                self.shoot();
                self.schedule_next_shot();
            }
            // reload screen
            (Screen::Reload, Action::Rotate(Direction::Clockwise)) => {
                self.secondary_gun.drum_clockwise();
            }
            (Screen::Reload, Action::Rotate(Direction::CounterClock)) => {
                self.secondary_gun.drum_counterclock();
            }
            (Screen::Reload, Action::Press(Button::Shoot)) => {
                self.secondary_gun.reload();
            }
            (Screen::Reload, Action::Press(Button::ReloadToggle)) => {
                self.screen = Screen::Normal;
            }
            // game over screen
            (Screen::Wasted, Action::Press(Button::Shoot)) => {
                self.screen = Screen::Start;
            }
            (Screen::Wasted, _) => {}
            // misc
            (_, Action::Release(Button::Shoot)) => {
                self.next_shot = None;
            }
            (_, Action::Release(_)) => (),
        }
    }
}
