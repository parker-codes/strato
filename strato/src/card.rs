use anyhow::Result;
use rand::Rng;
use thiserror::Error;

#[derive(PartialEq, Copy, Clone)]
pub struct Card {
    value: CardValue,
    flipped: bool,
}

impl Card {
    fn new(value: i32) -> Self {
        Self {
            value: CardValue::from(value),
            flipped: false,
        }
    }

    pub fn get_value(&self) -> Option<CardValue> {
        if self.is_flipped() {
            Some(self.value)
        } else {
            None
        }
    }

    pub fn flip(&mut self) {
        self.flipped = true;
    }

    pub fn is_flipped(&self) -> bool {
        self.flipped
    }
}

impl std::fmt::Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let flipped_marker = match self.is_flipped() {
            true => " ",
            false => "█",
        };

        write!(
            f,
            "[{flipped_marker}{: ^4}{flipped_marker}]",
            i32::from(self.value)
        )
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum CardValue {
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

#[derive(Debug, Clone, PartialEq)]
pub struct Deck(Vec<Card>);

impl Deck {
    pub const EMPTY_SIZE: usize = 0;
    pub const FULL_SIZE: usize = 150;

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

impl Default for Deck {
    /// Create a new deck which consists of ten full sets of -2 through 12.
    fn default() -> Self {
        let cards = (0..10)
            .map(|_| (-2..=12).map(|n| Card::new(n)).collect::<Vec<_>>())
            .flatten()
            .collect::<Vec<_>>();
        Self(cards)
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

#[derive(Error, Debug, PartialEq)]
pub enum SpreadActionError {
    #[error("Can't {0} card from a row that doesn't exist.")]
    RowDoesntExist(&'static str),
    #[error("Can't {0} card from a column that doesn't exist.")]
    ColumnDoesntExist(&'static str),
    #[error("No card found in that spot.")]
    NoCardFound,
    #[error("There is already a card in that spot.")]
    SpotTaken,
    #[error("That card has already been flipped.")]
    CardAlreadyFlipped,
}

type FourColumns = [Option<Card>; 4];
type ThreeByFourGrid = [FourColumns; 3];

#[derive(Default, Clone, PartialEq)]
pub struct PlayerSpread(ThreeByFourGrid);

impl PlayerSpread {
    /// Create a new deck which consists of ten full sets of -2 through 12.
    pub fn new() -> Self {
        Self([
            [None, None, None, None],
            [None, None, None, None],
            [None, None, None, None],
        ])
    }

    pub fn view(&self) -> Vec<Vec<Option<CardValue>>> {
        self.0
            .iter()
            .map(|row| {
                row.iter()
                    .map(|column| {
                        if let Some(card) = column {
                            if card.is_flipped() {
                                return Some(card.value);
                            }
                        }
                        // otherwise
                        None
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    }

    /// Take a card from a specified row and column.
    pub fn take_from(&mut self, row: usize, column: usize) -> Result<Card, SpreadActionError> {
        self.0
            .get_mut(row)
            .ok_or(SpreadActionError::RowDoesntExist("take"))?
            .get_mut(column)
            .ok_or(SpreadActionError::ColumnDoesntExist("take"))?
            .take()
            .ok_or(SpreadActionError::NoCardFound)
    }

    /// Put a card at a specified row and column.
    pub fn place_at(
        &mut self,
        card: Card,
        row: usize,
        column: usize,
    ) -> Result<(), SpreadActionError> {
        let place = self
            .0
            .get_mut(row)
            .ok_or(SpreadActionError::RowDoesntExist("place"))?
            .get_mut(column)
            .ok_or(SpreadActionError::ColumnDoesntExist("place"))?;

        if place.is_some() {
            return Err(SpreadActionError::SpotTaken);
        } else {
            place.replace(card);
            Ok(())
        }
    }

    /// Flip a card at a specified row and column.
    pub fn flip_at(&mut self, row: usize, column: usize) -> Result<(), SpreadActionError> {
        // Validates that row and column fit within bounds
        let selected_card = self
            .0
            .get_mut(row)
            .ok_or(SpreadActionError::RowDoesntExist("flip"))?
            .get_mut(column)
            .ok_or(SpreadActionError::ColumnDoesntExist("flip"))?
            .as_mut()
            .ok_or(SpreadActionError::NoCardFound)?;

        if selected_card.is_flipped() {
            return Err(SpreadActionError::CardAlreadyFlipped);
        } else {
            selected_card.flip();
            Ok(())
        }
    }

    /// Determine number of active columns.
    pub fn active_columns(&self) -> usize {
        self.0
            .get(0)
            .unwrap()
            .iter()
            .filter(|c| c.is_some())
            .collect::<Vec<_>>()
            .len()
    }

    /// If the column has matching cards, remove it.
    // TODO: Write tests for this
    pub fn remove_column_if_matches(&mut self, column: usize) -> Result<(), SpreadActionError> {
        let values = self
            .0
            .iter()
            .map(|row| row.get(column))
            .flatten()
            .collect::<Vec<_>>();

        // If any of the values are None, then the column is not full.
        if values.iter().any(|c| c.is_none()) {
            return Ok(());
        }
        // If any of the values are not flipped, then the column is not ready.
        if values.iter().any(|c| !c.unwrap().is_flipped()) {
            return Ok(());
        }

        let first_value = values.iter().next().unwrap().unwrap();
        let column_matches = values.iter().all(|c| c.unwrap() == first_value);

        if column_matches {
            // Remove column
            for row in self.0.iter_mut() {
                row[column] = None;
            }
        }

        Ok(())
    }

    pub fn remaining_cards(&self) -> impl Iterator<Item = &Card> {
        self.0
            .iter()
            .flatten()
            .filter(|c| c.is_some())
            .map(|c| c.as_ref().unwrap())
    }

    pub fn flipped_cards(&self) -> usize {
        self.remaining_cards().filter(|c| c.is_flipped()).count()
    }

    pub fn is_all_flipped(&self) -> bool {
        self.remaining_cards().all(|c| c.is_flipped())
    }

    pub fn score(&self) -> i32 {
        self.0
            .iter()
            .flatten()
            .filter(|c| c.is_some())
            .filter(|c| c.as_ref().unwrap().is_flipped())
            .map(|c| i32::from(c.as_ref().unwrap().value))
            .sum()
    }
}

impl std::fmt::Debug for PlayerSpread {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let cards = self
            .0
            .iter()
            .map(|row| {
                row.iter()
                    .map(|column| {
                        if let Some(column) = column {
                            format!("{column:?}")
                        } else {
                            // 8 empty spaces
                            "        ".to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("  ")
            })
            .collect::<Vec<_>>()
            .join("\n");
        write!(f, "\n{}", cards)
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

        // NOTE: We cannot cast as i32 or else it turns into 0
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
        assert!(!card.flipped);
    }

    #[test]
    fn can_determine_card_value() {
        let unflipped = Card::new(5);
        assert_eq!(unflipped.get_value(), None);

        let mut negative_two = Card::new(-2);
        negative_two.flip();
        assert_eq!(negative_two.get_value(), Some(CardValue::NegativeTwo));

        let mut zero = Card::new(0);
        zero.flip();
        assert_eq!(zero.get_value(), Some(CardValue::Zero));

        let mut twelve = Card::new(12);
        twelve.flip();
        assert_eq!(twelve.get_value(), Some(CardValue::Twelve));
    }

    #[test]
    fn can_initialize_deck_in_order() {
        let mut deck = Deck::default();
        assert_eq!(deck.draw(), Some(Card::new(12)));
    }

    #[test]
    fn deck_has_a_size() {
        let mut deck = Deck::default();
        assert_eq!(deck.size(), 150);
        deck.draw();
        deck.draw();
        deck.draw();
        assert_eq!(deck.size(), 147);
    }

    #[test]
    fn a_deck_can_be_depleted() {
        let mut deck = Deck::default();

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
        let mut deck = Deck::default();
        let snapshot = deck.clone();
        deck.shuffle();
        assert_eq!(deck.size(), 150);
        assert_ne!(deck, snapshot);
    }

    #[test]
    fn small_deck_can_be_shuffled() {
        let mut deck = Deck::default();

        for _ in 0..(deck.size() - 10) {
            deck.draw();
        }
        assert_eq!(deck.size(), 10);

        let snapshot = deck.clone();
        deck.shuffle();
        assert_ne!(deck, snapshot);
    }

    fn init_player_spread() -> PlayerSpread {
        let mut deck = Deck::default();
        deck.shuffle();

        let mut spread = PlayerSpread::new();

        for row in 0..3 {
            for column in 0..4 {
                let card = deck.draw().unwrap();
                spread.place_at(card, row, column).unwrap();
            }
        }

        spread
    }

    #[test]
    fn a_player_spread_can_provide_counts() {
        let mut spread = init_player_spread();

        assert_eq!(spread.flipped_cards(), 0);
        assert_eq!(spread.active_columns(), 4);

        spread.flip_at(0, 0).unwrap();
        spread.flip_at(1, 0).unwrap();
        spread.flip_at(2, 0).unwrap();

        assert_eq!(spread.flipped_cards(), 3);
    }

    #[test]
    fn a_filled_but_unflipped_player_spread_has_a_score_of_0() {
        let spread = init_player_spread();
        assert_eq!(spread.score(), 0);
    }

    #[test]
    fn a_player_spread_can_provide_a_score_1() {
        let mut spread = init_player_spread();
        assert_eq!(spread.score(), 0);

        let (row, column) = (1, 1);
        spread.take_from(row, column).unwrap(); // clear existing card

        let negative_two = Card::new(-2);
        spread.place_at(negative_two, row, column).unwrap(); // insert card
        spread.flip_at(row, column).unwrap(); // flip card

        assert_eq!(spread.score(), -2);
    }

    #[test]
    fn a_player_spread_can_provide_a_score_2() {
        let mut spread = PlayerSpread::new();
        assert_eq!(spread.score(), 0);

        let mut one = Card::new(1);
        one.flip();
        spread.place_at(one, 0, 0).unwrap();

        let mut five = Card::new(5);
        five.flip();
        spread.place_at(five, 1, 0).unwrap();

        let mut ten = Card::new(10);
        ten.flip();
        spread.place_at(ten, 2, 1).unwrap();

        let mut negative_one = Card::new(-1);
        negative_one.flip();
        spread.place_at(negative_one, 0, 3).unwrap();

        assert_eq!(spread.score(), 15);
    }
}
