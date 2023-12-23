use core::iter::Flatten;
use core::slice;

pub const FERRIS_OFFSET: u8 = 62;
pub const SPAWN_OFFSET: u8 = 128 - FERRIS_OFFSET;
pub const OPPONENT_HEIGHT: u8 = 21;

pub struct Lawn {
    opponents: [Option<Opponent>; 25],
    next_spawn: u8,
}

impl Default for Lawn {
    fn default() -> Self {
        Self {
            opponents: [
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None, None, None, None, None, None, None, None, None, None,
            ],
            next_spawn: 20,
        }
    }
}

struct Stats {
    spawn_rate: u8,
    cooldown: u8,
    concurrent: usize,
    speed: u32,
    health: u8,
}

impl Stats {
    pub fn from_score(score: u32) -> &'static Stats {
        match score {
            score if score < 3 => &Stats {
                spawn_rate: 10,
                cooldown: 5,
                concurrent: 1,
                speed: 15,
                health: 1,
            },
            score if score < 10 => &Stats {
                spawn_rate: 10,
                cooldown: 5,
                concurrent: 1,
                speed: 10,
                health: 1,
            },
            score if score < 20 => &Stats {
                spawn_rate: 10,
                cooldown: 5,
                concurrent: 1,
                speed: 10,
                health: 1,
            },
            _ => &Stats {
                spawn_rate: 10,
                cooldown: 5,
                concurrent: 1,
                speed: 10,
                health: 2,
            },
        }
    }
}

pub struct Opponent {
    x: u8,
    y: u8,
    speed: u32,
    next_step: u32,
    health: u8,
    cooldown: u8,
}

impl From<&Stats> for Opponent {
    fn from(stats: &Stats) -> Self {
        Self {
            x: SPAWN_OFFSET,
            y: 10, // TODO: random
            speed: stats.speed,
            next_step: 0,
            health: stats.health,
            cooldown: stats.cooldown,
        }
    }
}

impl Opponent {
    pub fn x(&self) -> u8 {
        self.x + FERRIS_OFFSET
    }

    pub fn y(&self) -> u8 {
        self.y
    }

    pub fn tick(&mut self) -> bool {
        self.next_step = self.next_step.saturating_sub(1);
        if self.next_step == 0 {
            self.x = self.x.saturating_sub(1);
            self.next_step = self.speed;
        }

        self.x == 0
    }

    pub fn hit(&mut self, y: u8) -> bool {
        if y >= self.y && y <= self.y + OPPONENT_HEIGHT {
            self.health = self.health.saturating_sub(1);
        }
        self.health == 0
    }
}

impl Lawn {
    pub fn tick(&mut self, score: u32) -> bool {
        let mut count = 0;
        for opp in self.opponents.iter_mut().flatten() {
            if opp.tick() {
                // game over
                return true;
            }
            count += 1;
        }

        self.next_spawn = self.next_spawn.saturating_sub(1);
        if self.next_spawn == 0 {
            let stats = Stats::from_score(score);
            if count < stats.concurrent {
                for slot in &mut self.opponents {
                    if slot.is_none() {
                        let opponent = Opponent::from(stats);
                        *slot = Some(opponent);
                        break;
                    }
                }
            }
            self.next_spawn = stats.spawn_rate;
        }

        false
    }

    pub fn shoot(&mut self, y: u8) -> bool {
        for slot in &mut self.opponents {
            if let Some(opp) = slot {
                if opp.hit(y) {
                    self.next_spawn += opp.cooldown;
                    *slot = None;
                    return true;
                }
            }
        }
        false
    }

    pub fn opponents(&self) -> Flatten<slice::Iter<'_, Option<Opponent>>> {
        self.opponents.iter().flatten()
    }
}
