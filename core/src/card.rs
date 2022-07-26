use rand::Rng;

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Card {
    value: CardValue,
    visible: bool,
}

impl Card {
    // TODO: make private so people can't fabricate cards
    pub fn new(value: i32) -> Self {
        Self {
            value: CardValue::from(value),
            visible: false,
        }
    }

    pub fn flip(&mut self) {
        self.visible = true;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum CardValue {
    NegativeTwo,
    NegativeOne,
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Eleven,
    Twelve,
}

impl From<i32> for CardValue {
    fn from(value: i32) -> Self {
        use CardValue::*;

        match value {
            -2 => NegativeTwo,
            -1 => NegativeOne,
            0 => Zero,
            1 => One,
            2 => Two,
            3 => Three,
            4 => Four,
            5 => Five,
            6 => Six,
            7 => Seven,
            8 => Eight,
            9 => Nine,
            10 => Ten,
            11 => Eleven,
            12 => Twelve,
            _ => panic!("Not a valid card value!"),
        }
    }
}

impl From<CardValue> for i32 {
    fn from(value: CardValue) -> Self {
        use CardValue::*;

        match value {
            NegativeTwo => -2,
            NegativeOne => -1,
            Zero => 0,
            One => 1,
            Two => 2,
            Three => 3,
            Four => 4,
            Five => 5,
            Six => 6,
            Seven => 7,
            Eight => 8,
            Nine => 9,
            Ten => 10,
            Eleven => 11,
            Twelve => 12,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Deck(Vec<Card>);

impl Deck {
    pub const EMPTY_SIZE: usize = 0;
    pub const FULL_SIZE: usize = 150;

    /// Create a new deck which consists of ten full sets of -2 through 12.
    pub fn new() -> Self {
        let cards = (0..10)
            .map(|_| (-2..=12).map(|n| Card::new(n)).collect::<Vec<_>>())
            .flatten()
            .collect::<Vec<_>>();
        Self(cards)
    }

    pub fn size(&self) -> usize {
        self.0.len()
    }

    /// Mimic human shuffle by splitting (sort of) in half and then zipping together (imperfectly), repeated
    /// a loose number of times. Then do some swaps until it feels right. 😄
    pub fn shuffle(&mut self) {
        let mut rng = rand::thread_rng();

        let times_to_shuffle = rng.gen_range(4..=7);
        let middle = self.size() / 2;
        let max_variance_from_middle = self.size() / 10;

        let mut left_hand = self.0.clone();

        for _ in 0..times_to_shuffle {
            let variance_from_middle = rng.gen_range(0..max_variance_from_middle);
            let guess_at_middle = if rng.gen_bool(1.0 / 2.0) {
                middle.checked_add(variance_from_middle).unwrap_or(middle)
            } else {
                middle.checked_sub(variance_from_middle).unwrap_or(middle)
            };

            let mut right_hand = left_hand.split_off(guess_at_middle);

            let mut shuffled: Vec<Card> = Vec::new();

            loop {
                if left_hand.is_empty() && right_hand.is_empty() {
                    break;
                }

                let left_cards_to_take = rng.gen_range(1..4);
                for _ in 0..left_cards_to_take {
                    if let Some(card) = left_hand.pop() {
                        shuffled.push(card);
                    }
                }

                let right_cards_to_take = rng.gen_range(1..4);
                for _ in 0..right_cards_to_take {
                    if let Some(card) = right_hand.pop() {
                        shuffled.push(card);
                    }
                }
            }

            left_hand = shuffled;
        }

        let number_of_swaps = rng.gen_range(4..12);
        for _ in 0..number_of_swaps {
            let first = rng.gen_range(0..self.size());
            let second = rng.gen_range(0..self.size());
            left_hand.swap(first, second);
        }

        self.0 = left_hand;
    }

    /// Draw a card from the top of the deck.
    pub fn draw(&mut self) -> Option<Card> {
        self.0.pop()
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct DiscardPile(Vec<Card>);

impl DiscardPile {
    /// Create a new, empty pile.
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn size(&self) -> usize {
        self.0.len()
    }

    /// Take a card from the top of the discard pile.
    pub fn take(&mut self) -> Option<Card> {
        self.0.pop()
    }

    /// Put a card on the top of the discard pile.
    pub fn put(&mut self, card: Card) {
        self.0.push(card)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cards_have_value() {
        assert_eq!(CardValue::from(-2), CardValue::NegativeTwo);
        assert_eq!(CardValue::from(0), CardValue::Zero);
        assert_eq!(CardValue::from(4), CardValue::Four);
        assert_eq!(CardValue::from(12), CardValue::Twelve);

        assert_eq!(i32::from(CardValue::NegativeTwo), -2);
        assert_eq!(i32::from(CardValue::Zero), 0);
        assert_eq!(i32::from(CardValue::Ten), 10);
    }

    #[test]
    #[should_panic]
    #[allow(unused_must_use)]
    fn card_must_fit_in_valid_range() {
        CardValue::from(-3);
    }

    #[test]
    fn card_has_value_and_starts_hidden() {
        let card = Card::new(5);
        assert_eq!(card.value, CardValue::Five);
        assert!(!card.visible);
    }

    #[test]
    fn can_initialize_deck_in_order() {
        let mut deck = Deck::new();
        assert_eq!(deck.draw(), Some(Card::new(12)));
    }

    #[test]
    fn deck_has_a_size() {
        let mut deck = Deck::new();
        assert_eq!(deck.size(), 150);
        deck.draw();
        deck.draw();
        deck.draw();
        assert_eq!(deck.size(), 147);
    }

    #[test]
    fn a_deck_can_be_depleted() {
        let mut deck = Deck::new();

        for _ in 1..deck.size() {
            deck.draw();
        }

        let last_card = deck.draw();
        assert_eq!(last_card, Some(Card::new(-2)));

        // now depleted
        assert_eq!(deck.draw(), None);
        assert_eq!(deck.size(), 0);
    }

    #[test]
    fn deck_can_be_shuffled() {
        let mut deck = Deck::new();
        let snapshot = deck.clone();
        deck.shuffle();
        assert_eq!(deck.size(), 150);
        assert_ne!(deck, snapshot);
    }

    #[test]
    fn small_deck_can_be_shuffled() {
        let mut deck = Deck::new();

        for _ in 0..(deck.size() - 10) {
            deck.draw();
        }
        assert_eq!(deck.size(), 10);

        let snapshot = deck.clone();
        deck.shuffle();
        assert_ne!(deck, snapshot);
    }
}
