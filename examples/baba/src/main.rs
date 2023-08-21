use game::Level;
use goodman::prelude::*;
use other::{AllCharacterData, Move, NounPropCombi, Object, Property, VecPos};

mod game;
mod other;

use rodio::{Decoder, Source};
use std::fs::File;
use std::io::BufReader;

pub const WINDOW_SIZE: Vec2 = vec2(1200., 750.); //1500x1000
const GRID_SIZE: (usize, usize) = (20, 14);

fn main() {
    block_on(run());
}

async fn run() {
    let event_loop = EventLoop::new();

    let mut engine = Engine::new(WINDOW_SIZE, &event_loop, true).await;
    engine.set_target_fps(Some(144));
    // engine.set_target_tps(Some(300 * 1000));
    // engine.enable_feature(Feature::EngineUi);

    let game = Game::new(&mut engine);

    engine.enter_loop(game, event_loop);
}

pub struct Game {
    grid: Vec<Vec<Object>>,
    character_data: AllCharacterData,
    noun_prop_combi: Vec<NounPropCombi>,
    current_level: Level,
    textures: Vec<Texture>,
    source: rodio::source::Buffered<Decoder<BufReader<File>>>,
    #[allow(dead_code)]
    stream: rodio::OutputStream,
    stream_handle: rodio::OutputStreamHandle,
}
impl Manager for Game {
    fn new(engine: &mut Engine) -> Self {
        // The following two lines change the working directory, but you won't need to do this
        let root = std::path::Path::new("/home/wurmwillem/Programming/Goodman/examples/baba");
        std::env::set_current_dir(&root).unwrap();

        // Change this to the location of the assets folder on your pc
        let path_to_assets_folder =
            "/home/wurmwillem/Programming/Goodman/examples/baba/src/assets/";
        let rest_of_path_vec = vec![
            "floor.png",
            "is.png",
            "baba.png",
            "baba c.png",
            "you.png",
            "flag.png",
            "flag c.png",
            "win.png",
            "wall.png",
            "wall c.png",
            "stop.png",
        ];

        let textures = engine
            .create_texture_vec(path_to_assets_folder, &rest_of_path_vec)
            .unwrap();

        let mut grid = vec![vec![]];
        let current_level = Level::Level1;
        current_level.load_level(&mut grid);

        let file = BufReader::new(File::open("src/assets/pop.flac").unwrap());
        let source = Decoder::new(file).unwrap().buffered();

        let (stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

        Self {
            grid,
            character_data: AllCharacterData::new(),
            noun_prop_combi: vec![],
            current_level,
            textures,
            source,
            stream,
            stream_handle,
        }
    }

    fn start(&mut self) {
        self.update_character_data();
    }

    fn update(&mut self, _delta_t: f64, input: &Input) {
        let mut where_to_move = (0, 0);
        if input.is_button_pressed(Button::W) {
            where_to_move.1 = -1;
        }
        if input.is_button_pressed(Button::D) {
            where_to_move.0 = 1;
        }
        if input.is_button_pressed(Button::S) {
            where_to_move.1 = 1;
        }
        if input.is_button_pressed(Button::A) {
            where_to_move.0 = -1;
        }
        if input.is_button_pressed(Button::One) {
            self.current_level = Level::Level1;
            self.current_level.load_level(&mut self.grid);
        }
        if input.is_button_pressed(Button::Two) {
            self.current_level = Level::Level2;
            self.current_level.load_level(&mut self.grid);
        }

        let mut moves: Vec<Move> = vec![];
        for j in 0..self.grid.len() {
            for i in 0..self.grid[0].len() {
                if let Object::Character(char) = self.grid[j][i] {
                    if self.character_data.is_you(char) {
                        if where_to_move == (0, 0) {
                            continue;
                        }

                        let i_j: VecPos = VecPos::new((i, j));

                        let mut should_continue = false;
                        for m in &moves {
                            if m.to == i_j {
                                should_continue = true;
                                break;
                            }
                        }
                        if should_continue {
                            continue;
                        }

                        let next_grid_pos = VecPos::add_i32_tuple(i_j, where_to_move);
                        let mov = Move::new(i_j, next_grid_pos);
                        let mut moves_to_make = vec![mov];

                        loop {
                            let to = moves_to_make[moves_to_make.len() - 1].to;
                            if self.grid.get(to.j).is_none() || self.grid[to.j].get(to.i).is_none()
                            {
                                break;
                            }

                            if self.grid[to.j][to.i] == Object::Empty {
                                for m in moves_to_make.iter().rev() {
                                    moves.push(*m);
                                }
                                break;
                            } else {
                                let from = to;
                                let to = VecPos::add_i32_tuple(from, where_to_move);
                                if let Object::Character(char) = self.grid[from.j][from.i] {
                                    if self.character_data.get_if_enabled(
                                        char.get_corresponding_noun(),
                                        Property::Win,
                                    ) {
                                        self.win();
                                    }
                                }
                                if let Object::Character(char) = self.grid[from.j][from.i] {
                                    if self.character_data.get_if_enabled(
                                        char.get_corresponding_noun(),
                                        Property::Stop,
                                    ) {
                                        break;
                                    }
                                }
                                moves_to_make.push(Move::new(from, to));
                            }
                        }
                    }
                }
            }
        }
        for mov in &moves {
            if self.grid[mov.from.j][mov.from.i] != Object::Empty {
                self.move_object(*mov);
            }
        }

        if !moves.is_empty() {
            self.update_character_data();
        }
    }

    fn render(&self, engine: &mut Engine) {
        let size = vec2(
            WINDOW_SIZE.x / self.grid[0].len() as f64,
            WINDOW_SIZE.y / self.grid.len() as f64,
        );

        for j in 0..self.grid.len() {
            for i in 0..self.grid[0].len() {
                let pos = vec2(i as f64 * size.x, j as f64 * size.y);
                engine.render_texture(&rect_vec(pos, size), &self.textures[0]);

                if self.grid[j][i] != Object::Empty {
                    let index = self.grid[j][i].get_tex_index();
                    engine.render_texture(&rect_vec(pos, size), &self.textures[index]);
                };
            }
        }
    }
}
