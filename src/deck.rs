use crate::AttackType;
use rand::seq::SliceRandom;
use rand::thread_rng;

const HAND_LIMIT: usize = 7;

pub struct Deck {
    cards: Vec<AttackType>,
    discard: Vec<AttackType>,
    pub hand: Vec<AttackType>,
    pub selected: i32,
}

impl Deck {
    pub fn new(cards: Vec<AttackType>) -> Self {
        Deck {
            cards,
            discard: Vec::new(),
            hand: Vec::new(),
            selected: -1,
        }
    }

    pub fn draw(&mut self) {
        if self.hand.len() >= HAND_LIMIT {
            return;
        }

        if self.cards.len() == 0 {
            self.shuffle();
        }

        // draw can be empty if both the discard and library are empty
        let draw = self.cards.pop();
        if let Some(draw) = draw {
            self.hand.push(draw);
        }
    }

    pub fn discard_selected(&mut self) {
        if self.selected < 0 || (self.selected as usize) >= self.hand.len() {
            return;
        }

        let card = self.hand.remove(self.selected as usize);
        self.discard.push(card);
        self.selected = -1;
    }

    pub fn cards_remaining(&self) -> i32 {
        self.cards.len() as i32
    }

    pub fn cards_discarded(&self) -> i32 {
        self.discard.len() as i32
    }

    fn shuffle(&mut self) {
        for card in self.discard.drain(..) {
            self.cards.push(card);
        }

        self.cards.shuffle(&mut thread_rng());
    }
}
