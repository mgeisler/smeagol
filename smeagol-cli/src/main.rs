use smeagol_cli::{views, State};

fn main() {
    let (term_width, term_height) = termion::terminal_size().unwrap();
    let mut siv = cursive::Cursive::default();

    let life = smeagol::Life::from_rle_file(std::env::args().nth(1).unwrap()).unwrap();
    let state = State::new_centered(life, term_width as u64, term_height as u64);

    siv.add_fullscreen_layer(views::main_view(&state));

    for key_command in smeagol_cli::key_commands(&state) {
        key_command.register(&mut siv);
    }

    smeagol_cli::start_smeagol_thread(&mut siv, &state);

    siv.run();
}
