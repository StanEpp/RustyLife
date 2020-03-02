mod game_of_life;


fn main() {
    let mut gof = game_of_life::GameOfLife::new((32, 32),
                                                "Game of Life",
                                                (1920, 1080));
    gof.run();
}