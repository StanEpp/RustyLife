mod game_of_life;


fn main() {
    let mut gof = game_of_life::GameOfLife::new((800, 600),
                                                String::from("Game of Life"),
                                                (1920, 1080));
    gof.run();
}

