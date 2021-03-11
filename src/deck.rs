use crate::AttackType;
use rand::seq::SliceRandom;
use rand::thread_rng;

const HAND_LIMIT: usize = 7;

pub struct Deck {
    cards: Vec<AttackType>,
    pub hand: Vec<AttackType>,
    pub selected: i32,
    next_draw: usize,
}

impl Deck {
    pub fn new(cards: Vec<AttackType>) -> Self {
        Deck {
            cards,
            hand: Vec::new(),
            selected: -1,
            next_draw: 0,
        }
    }

    pub fn draw(&mut self) {
        if self.hand.len() >= HAND_LIMIT {
            return;
        }

        self.next_draw += 1;
        if self.next_draw >= self.cards.len() {
            self.shuffle();
        }

        let draw = self.cards[self.next_draw];
        self.hand.push(draw);
    }

    pub fn cards_remaining(&self) -> i32 {
        (self.cards.len() - self.next_draw) as i32
    }

    fn shuffle(&mut self) {
        self.cards.shuffle(&mut thread_rng());
        self.next_draw = 0;
    }
}
