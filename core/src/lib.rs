pub struct StratoGame {
    pub state: GameState,
    pub context: GameContext,
}

impl StratoGame {
    pub fn new() -> Self {
        Self {
            state: GameState::default(),
            context: GameContext::default(),
        }
    }

    pub fn add_player(&mut self, player: Player) {
        if self.state == GameState::WaitingForPlayers {
            self.context.players.push(player);
        }
    }

    pub fn list_players(&self) -> Vec<Player> {
        self.context.players.clone()
    }

    pub fn start(&mut self) {
        if self.state == GameState::WaitingForPlayers && self.context.players.len() > 0 {
            self.state = GameState::Active;
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub enum GameState {
    #[default]
    WaitingForPlayers,
    Active,
    Ended,
}

#[derive(Debug, Default)]
pub struct GameContext {
    pub players: Vec<Player>,
}

#[derive(Debug, Default, PartialEq, Copy, Clone)]
pub struct Player {
    pub name: &'static str,
}

impl Player {
    pub fn new(name: &'static str) -> Self {
        Self { name }
    }
}

#[derive(Debug, PartialEq)]
struct Card {
    value: CardValue,
    visible: bool,
}

impl Card {
    fn new(value: i32) -> Self {
        Self {
            value: CardValue::from(value),
            visible: false,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
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

struct Deck(Vec<Card>);

impl Deck {
    /// Create a new deck which consists of ten full sets of -2 through 12.
    fn new() -> Self {
        let cards = (0..10)
            .map(|_| (-2..=12).map(|n| Card::new(n)).collect::<Vec<_>>())
            .flatten()
            .collect::<Vec<_>>();
        Self(cards)
    }

    fn size(&self) -> usize {
        self.0.len()
    }

    /// Draw a card from the top of the deck.
    fn draw(&mut self) -> Option<Card> {
        self.0.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_game_can_be_initialized() {
        let game = StratoGame::new();
        assert_eq!(game.state, GameState::WaitingForPlayers);
    }

    #[test]
    fn players_can_be_added() {
        let mut game = StratoGame::new();
        let player_1 = Player::new("Parker");
        game.add_player(player_1);
        assert_eq!(game.state, GameState::WaitingForPlayers);
    }

    #[test]
    fn a_game_can_be_started() {
        let mut game = StratoGame::new();
        let player_1 = Player::new("Parker");
        game.add_player(player_1);
        game.start();
        assert_eq!(game.state, GameState::Active);
    }

    #[test]
    fn cant_start_without_players() {
        let mut game = StratoGame::new();
        game.start();
        assert_eq!(game.state, GameState::WaitingForPlayers);
    }

    #[test]
    fn starting_multiple_times_is_inconsequential() {
        let mut game = StratoGame::new();
        let player_1 = Player::new("Parker");
        game.add_player(player_1);
        game.start();
        game.start();
        game.start();
        game.start();
        game.start();
        assert_eq!(game.state, GameState::Active);
    }

    #[test]
    pub fn can_list_players() {
        let mut game = StratoGame::new();
        let player_1 = Player::new("Parker");
        game.add_player(player_1);
        let player_2 = Player::new("Lexi");
        game.add_player(player_2);
        assert_eq!(game.list_players(), vec![player_1, player_2]);
    }

    #[test]
    pub fn cant_change_players_after_game_starts() {
        let mut game = StratoGame::new();
        let player_1 = Player::new("Parker");
        game.add_player(player_1);
        let player_2 = Player::new("Lexi");
        game.add_player(player_2);
        game.start();
        assert_eq!(game.state, GameState::Active);

        let player_3 = Player::new("Trevor");
        game.add_player(player_3);
        assert_eq!(game.list_players(), vec![player_1, player_2]);
    }

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
}
