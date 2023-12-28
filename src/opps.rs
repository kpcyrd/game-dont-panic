use crate::gfx;
use crate::guns;
use core::iter::Flatten;
use core::slice;
use rand_core::RngCore;

pub const SPAWN_OFFSET_X: u8 = gfx::SCREEN_WIDTH - gfx::FERRIS_OFFSET;
pub const MAX_SPAWN_Y: u8 = gfx::min(
    gfx::SCREEN_HEIGHT - gfx::OPPONENT_HEIGHT,
    gfx::FERRIS_MAX_Y + guns::MAX_GUARANTEED_REACH,
);
pub const HIT_PUSHBACK: u8 = 5;

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
        if score < 3 {
            &Stats {
                spawn_rate: 10,
                cooldown: 5,
                concurrent: 1,
                speed: 15,
                health: 1,
            }
        } else if score < 10 {
            &Stats {
                spawn_rate: 10,
                cooldown: 5,
                concurrent: 1,
                speed: 10,
                health: 1,
            }
        } else if score < 35 {
            &Stats {
                spawn_rate: 10,
                cooldown: 5,
                concurrent: 1,
                speed: 10,
                health: 2,
            }
        } else if score < 50 {
            &Stats {
                spawn_rate: 7,
                cooldown: 5,
                concurrent: 1,
                speed: 5,
                health: 2,
            }
        } else {
            &Stats {
                spawn_rate: 5,
                cooldown: 3,
                concurrent: 1,
                speed: 5,
                health: 3,
            }
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

impl Opponent {
    fn create<R: RngCore>(stats: &Stats, mut random: R) -> Self {
        let mut y = [0u8];
        random.fill_bytes(&mut y);
        Self {
            x: SPAWN_OFFSET_X,
            y: y[0] % MAX_SPAWN_Y,
            speed: stats.speed,
            next_step: 0,
            health: stats.health,
            cooldown: stats.cooldown,
        }
    }
}

impl Opponent {
    pub fn x(&self) -> u8 {
        self.x + gfx::FERRIS_OFFSET
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
        if y >= self.y && y <= self.y + gfx::OPPONENT_HEIGHT {
            self.health = self.health.saturating_sub(1);
            self.x = u8::min(self.x.saturating_add(HIT_PUSHBACK), SPAWN_OFFSET_X);
        }
        self.health == 0
    }
}

impl Lawn {
    pub fn tick<R: RngCore>(&mut self, score: u32, random: R) -> bool {
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
                        let opponent = Opponent::create(stats, random);
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
