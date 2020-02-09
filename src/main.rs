mod game_of_life;


fn main() {
    let mut gof = game_of_life::GameOfLife::new((1001, 1001),
                                                String::from("Game of Life"),
                                                (1920, 1080));
    gof.run();
}

