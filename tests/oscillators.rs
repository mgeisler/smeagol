/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

const REPS: usize = 10;

fn oscillate(life: &mut smeagol::Life, period: usize) {
    life.set_step_log_2(0);

    let before = life.get_alive_cells();

    for _ in 0..REPS {
        for _ in 0..period {
            life.step();
        }

        let after = life.get_alive_cells();

        assert_eq!(before, after);
    }
}

fn oscillate_jump(life: &mut smeagol::Life, period: usize) {
    let before = life.get_alive_cells();
    life.set_step_log_2(10);
    for _ in 0..period {
        life.step();
    }
    let after = life.get_alive_cells();
    assert_eq!(before, after);
}

#[test]
fn figure_eight() {
    let mut life = smeagol::Life::from_rle_file("./assets/figureeight.rle").unwrap();
    oscillate(&mut life, 8);
    oscillate_jump(&mut life, 8);
}

#[test]
fn pentadecathlon() {
    let mut life = smeagol::Life::from_rle_file("./assets/pentadecathlon.rle").unwrap();
    oscillate(&mut life, 15);
    oscillate_jump(&mut life, 15);
}

#[test]
fn pulsar() {
    let mut life = smeagol::Life::from_rle_file("./assets/pulsar.rle").unwrap();
    oscillate(&mut life, 3);
    oscillate_jump(&mut life, 3);
}

#[test]
fn queen_bee_shuttle() {
    let mut life = smeagol::Life::from_rle_file("./assets/queenbeeshuttle.rle").unwrap();
    oscillate(&mut life, 30);
    oscillate_jump(&mut life, 30);
}
