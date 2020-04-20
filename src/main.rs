mod rusty_life;

fn main() {
    let mut gof = rusty_life::RustyLife::new((1000, 1000),
                                             "Rusty Life",
                                             (1920, 1080));
    gof.run();
}