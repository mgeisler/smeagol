fn main() {
    let mut life = smeagol::Life::from_macrocell_file("./smeagol/assets/waterbear.mc").unwrap();
    loop {
        life.step();
        println!("{}", life.generation());
    }
}
