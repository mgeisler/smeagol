#[macro_use]
extern crate failure;
#[macro_use]
extern crate nom;
#[macro_use]
extern crate packed_simd;

mod evolve;
mod life;
pub mod parse;

pub use self::life::*;
use self::parse::*;

use packed_simd::u16x16;
use std::rc::Rc;

fn center_of_four_u16x16(
    nw_grid: u16x16,
    ne_grid: u16x16,
    sw_grid: u16x16,
    se_grid: u16x16,
) -> u16x16 {
    let nw_grid = nw_grid << 8;
    let sw_grid = sw_grid << 8;
    let left: u16x16 = shuffle!(
        nw_grid,
        sw_grid,
        [8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23]
    );

    let ne_grid = ne_grid >> 8;
    let se_grid = se_grid >> 8;
    let right: u16x16 = shuffle!(
        ne_grid,
        se_grid,
        [8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23]
    );

    left | right
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "IO error: {}", io)]
    Io { io: std::io::Error },
    #[fail(display = "RLE pattern error: {}", rle)]
    Rle { rle: RleError },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Cell {
    Alive,
    Dead,
}

impl Cell {
    pub fn new(alive: bool) -> Self {
        if alive {
            Cell::Alive
        } else {
            Cell::Dead
        }
    }

    pub fn is_alive(self) -> bool {
        match self {
            Cell::Alive => true,
            Cell::Dead => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Position {
    pub x: i64,
    pub y: i64,
}

impl Position {
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    pub fn offset(&self, x_offset: i64, y_offset: i64) -> Self {
        Self {
            x: self.x + x_offset,
            y: self.y + y_offset,
        }
    }

    pub fn quadrant(&self) -> Quadrant {
        match (self.x < 0, self.y < 0) {
            (true, true) => Quadrant::Northwest,
            (false, true) => Quadrant::Northeast,
            (true, false) => Quadrant::Southwest,
            (false, false) => Quadrant::Southeast,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BoundingBox {
    upper_left: Position,
    lower_right: Position,
}

impl BoundingBox {
    pub fn new(upper_left: Position, lower_right: Position) -> Self {
        assert!(upper_left.x <= lower_right.x);
        assert!(upper_left.y <= lower_right.y);
        Self {
            upper_left,
            lower_right,
        }
    }

    pub fn upper_left(&self) -> Position {
        self.upper_left
    }

    pub fn lower_right(&self) -> Position {
        self.lower_right
    }

    pub fn combine(&self, other: BoundingBox) -> Self {
        let min_x = Ord::min(self.upper_left.x, other.upper_left.x);
        let min_y = Ord::min(self.upper_left.y, other.upper_left.y);
        let max_x = Ord::max(self.lower_right.x, other.lower_right.x);
        let max_y = Ord::max(self.lower_right.y, other.lower_right.y);

        Self::new(Position::new(min_x, min_y), Position::new(max_x, max_y))
    }

    pub fn offset(&self, x_offset: i64, y_offset: i64) -> Self {
        Self::new(
            self.upper_left.offset(x_offset, y_offset),
            self.lower_right.offset(x_offset, y_offset),
        )
    }

    pub fn pad(&self, amount: i64) -> Self {
        assert!(amount >= 0);
        Self {
            upper_left: self.upper_left.offset(-amount, -amount),
            lower_right: self.lower_right.offset(amount, amount),
        }
    }
}

pub enum Quadrant {
    Northeast,
    Northwest,
    Southeast,
    Southwest,
}

pub struct NodeTemplate {
    pub nw: Rc<Node>,
    pub ne: Rc<Node>,
    pub sw: Rc<Node>,
    pub se: Rc<Node>,
}

#[derive(Clone)]
pub struct Store {
    indices: hashbrown::HashMap<NodeKind, usize>,
    nodes: Vec<Rc<Node>>,
    empties: Vec<Rc<Node>>,
    steps: Vec<Option<Rc<Node>>>,
    jumps: Vec<Option<Rc<Node>>>,
    step_log_2: u8,
}
impl Store {
    pub fn new() -> Self {
        Self {
            indices: hashbrown::HashMap::default(),
            nodes: vec![],
            empties: vec![],
            steps: vec![],
            jumps: vec![],
            step_log_2: 0,
        }
    }

    pub fn step_log_2(&self) -> u8 {
        self.step_log_2
    }

    pub fn set_step_log_2(&mut self, step_log_2: u8) {
        if step_log_2 != self.step_log_2 {
            self.step_log_2 = step_log_2;
            self.steps = vec![None; self.steps.len()]
        }
    }

    pub fn get_step(&self, node: &Node) -> Option<Rc<Node>> {
        self.steps[node.index.unwrap()].clone()
    }

    pub fn add_step(&mut self, node: &Node, step: Rc<Node>) {
        self.steps[node.index.unwrap()] = Some(step);
    }

    pub fn get_jump(&self, node: &Node) -> Option<Rc<Node>> {
        self.jumps[node.index.unwrap()].clone()
    }

    pub fn add_jump(&mut self, node: &Node, jump: Rc<Node>) {
        self.jumps[node.index.unwrap()] = Some(jump);
    }
}

impl Store {
    pub fn create_empty(&mut self, level: u8) -> Rc<Node> {
        if level < self.empties.len() as u8 {
            return self.empties[level as usize].clone();
        }
        let empty = if level == 4 {
            self.create_leaf(u16x16::splat(0))
        } else {
            let empty = self.create_empty(level - 1);
            self.create_interior(NodeTemplate {
                nw: empty.clone(),
                ne: empty.clone(),
                sw: empty.clone(),
                se: empty.clone(),
            })
        };
        self.empties.push(empty.clone());
        empty
    }

    pub fn create_leaf(&mut self, grid: u16x16) -> Rc<Node> {
        let leaf = Node {
            kind: NodeKind::Leaf { grid },
            index: None,
            level: 4,
            population: u128::from(grid.count_ones().wrapping_sum()),
        };
        self.add_node(leaf)
    }

    pub fn create_interior(&mut self, template: NodeTemplate) -> Rc<Node> {
        let interior = Node {
            level: template.nw.level + 1,
            population: template.nw.population
                + template.ne.population
                + template.sw.population
                + template.se.population,
            kind: NodeKind::Interior {
                nw: template.nw,
                ne: template.ne,
                sw: template.sw,
                se: template.se,
            },
            index: None,
        };
        self.add_node(interior)
    }

    fn add_node(&mut self, mut node: Node) -> Rc<Node> {
        if let Some(&index) = self.indices.get(&node.kind) {
            self.nodes[index].clone()
        } else {
            let index = self.nodes.len();
            node.index = Some(index);
            self.indices.insert(node.kind.clone(), index);
            let rc = Rc::new(node);
            self.nodes.push(rc.clone());
            self.jumps.push(None);
            self.steps.push(None);
            rc
        }
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum NodeKind {
    Leaf {
        grid: u16x16,
    },
    Interior {
        nw: Rc<Node>,
        ne: Rc<Node>,
        sw: Rc<Node>,
        se: Rc<Node>,
    },
}

#[derive(Clone)]
pub struct Node {
    kind: NodeKind,
    index: Option<usize>,
    level: u8,
    population: u128,
}

impl Eq for Node {}

impl std::hash::Hash for Node {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        match &self.kind {
            NodeKind::Leaf { grid } => grid.hash(state),
            NodeKind::Interior { nw, ne, sw, se } => {
                nw.index.hash(state);
                ne.index.hash(state);
                sw.index.hash(state);
                se.index.hash(state);
            }
        }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        match (&self.kind, &other.kind) {
            (NodeKind::Leaf { grid }, NodeKind::Leaf { grid: other_grid }) => grid == other_grid,
            (
                NodeKind::Interior { nw, ne, sw, se },
                NodeKind::Interior {
                    nw: other_nw,
                    ne: other_ne,
                    sw: other_sw,
                    se: other_se,
                },
            ) => {
                nw.index == other_nw.index
                    && ne.index == other_ne.index
                    && sw.index == other_sw.index
                    && se.index == other_se.index
            }
            _ => false,
        }
    }
}

impl Node {
    fn grid(&self) -> u16x16 {
        match &self.kind {
            NodeKind::Leaf { grid } => *grid,
            NodeKind::Interior { .. } => panic!(),
        }
    }
}

impl Node {
    pub fn expand(&self, store: &mut Store) -> Rc<Node> {
        match &self.kind {
            NodeKind::Leaf { .. } => panic!(),
            NodeKind::Interior { nw, ne, sw, se } => {
                let empty = store.create_empty(self.level - 1);

                let nw = store.create_interior(NodeTemplate {
                    nw: empty.clone(),
                    ne: empty.clone(),
                    sw: empty.clone(),
                    se: nw.clone(),
                });

                let ne = store.create_interior(NodeTemplate {
                    nw: empty.clone(),
                    ne: empty.clone(),
                    sw: ne.clone(),
                    se: empty.clone(),
                });

                let sw = store.create_interior(NodeTemplate {
                    nw: empty.clone(),
                    ne: sw.clone(),
                    sw: empty.clone(),
                    se: empty.clone(),
                });

                let se = store.create_interior(NodeTemplate {
                    nw: se.clone(),
                    ne: empty.clone(),
                    sw: empty.clone(),
                    se: empty,
                });

                store.create_interior(NodeTemplate { nw, ne, sw, se })
            }
        }
    }

    pub fn nw(&self) -> Rc<Node> {
        match &self.kind {
            NodeKind::Leaf { .. } => panic!(),
            NodeKind::Interior { nw, .. } => nw.clone(),
        }
    }

    pub fn ne(&self) -> Rc<Node> {
        match &self.kind {
            NodeKind::Leaf { .. } => panic!(),
            NodeKind::Interior { ne, .. } => ne.clone(),
        }
    }

    pub fn sw(&self) -> Rc<Node> {
        match &self.kind {
            NodeKind::Leaf { .. } => panic!(),
            NodeKind::Interior { sw, .. } => sw.clone(),
        }
    }

    pub fn se(&self) -> Rc<Node> {
        match &self.kind {
            NodeKind::Leaf { .. } => panic!(),
            NodeKind::Interior { se, .. } => se.clone(),
        }
    }

    pub fn center_subnode(&self, store: &mut Store) -> Rc<Node> {
        match &self.kind {
            NodeKind::Leaf { .. } => panic!(),
            NodeKind::Interior { nw, ne, sw, se } => {
                if self.level == 5 {
                    let nw_grid = nw.grid();
                    let ne_grid = ne.grid();
                    let sw_grid = sw.grid();
                    let se_grid = se.grid();
                    store.create_leaf(center_of_four_u16x16(nw_grid, ne_grid, sw_grid, se_grid))
                } else {
                    let template = NodeTemplate {
                        nw: nw.se(),
                        ne: ne.sw(),
                        sw: Node::ne(sw),
                        se: se.nw(),
                    };
                    store.create_interior(template)
                }
            }
        }
    }

    pub fn north_subsubnode(&self, store: &mut Store) -> Rc<Node> {
        let w = self.nw();
        let e = self.ne();
        centered_horiz(store, w, e)
    }

    pub fn south_subsubnode(&self, store: &mut Store) -> Rc<Node> {
        let w = self.sw();
        let e = self.se();
        centered_horiz(store, w, e)
    }

    pub fn west_subsubnode(&self, store: &mut Store) -> Rc<Node> {
        let n = self.nw();
        let s = self.sw();
        centered_vert(store, n, s)
    }

    pub fn east_subsubnode(&self, store: &mut Store) -> Rc<Node> {
        let n = self.ne();
        let s = self.se();
        centered_vert(store, n, s)
    }
}

fn centered_horiz(store: &mut Store, w: Rc<Node>, e: Rc<Node>) -> Rc<Node> {
    match (&w.kind, &e.kind) {
        (NodeKind::Leaf { .. }, NodeKind::Leaf { .. }) => panic!(),
        (
            NodeKind::Interior {
                ne: w_ne, se: w_se, ..
            },
            NodeKind::Interior {
                nw: e_nw, sw: e_sw, ..
            },
        ) => {
            if w.level == 5 {
                let nw_grid = w_ne.grid();
                let ne_grid = e_nw.grid();
                let sw_grid = w_se.grid();
                let se_grid = e_sw.grid();
                store.create_leaf(center_of_four_u16x16(nw_grid, ne_grid, sw_grid, se_grid))
            } else {
                let nw = w_ne.se();
                let ne = e_nw.sw();
                let sw = Node::ne(w_se);
                let se = e_sw.nw();
                store.create_interior(NodeTemplate { nw, ne, sw, se })
            }
        }
        _ => unreachable!(),
    }
}

fn centered_vert(store: &mut Store, n: Rc<Node>, s: Rc<Node>) -> Rc<Node> {
    match (&n.kind, &s.kind) {
        (NodeKind::Leaf { .. }, NodeKind::Leaf { .. }) => panic!(),
        (
            NodeKind::Interior {
                sw: n_sw, se: n_se, ..
            },
            NodeKind::Interior {
                nw: s_nw, ne: s_ne, ..
            },
        ) => {
            if n.level == 5 {
                let nw_grid = n_sw.grid();
                let ne_grid = n_se.grid();
                let sw_grid = s_nw.grid();
                let se_grid = s_ne.grid();
                store.create_leaf(center_of_four_u16x16(nw_grid, ne_grid, sw_grid, se_grid))
            } else {
                let nw = n_sw.se();
                let ne = n_se.sw();
                let sw = Node::ne(s_nw);
                let se = s_ne.nw();
                store.create_interior(NodeTemplate { nw, ne, sw, se })
            }
        }
        _ => unreachable!(),
    }
}

impl Node {
    pub fn min_coord(&self) -> i64 {
        if self.level == 64 {
            i64::min_value()
        } else {
            -(1 << (self.level - 1))
        }
    }

    pub fn max_coord(&self) -> i64 {
        if self.level == 64 {
            i64::max_value()
        } else {
            (1 << (self.level - 1)) - 1
        }
    }
}

impl Node {
    pub fn get_cell(&self, pos: Position) -> Cell {
        match &self.kind {
            NodeKind::Leaf { grid } => {
                let x_offset = (7 - pos.x) as usize;
                let y_offset = (pos.y + 8) as usize;
                Cell::new(grid.extract(y_offset) & (1 << x_offset) > 0)
            }
            NodeKind::Interior { nw, ne, sw, se } => {
                // quarter side length
                let offset = 1 << (self.level - 2);

                match pos.quadrant() {
                    Quadrant::Northwest => nw.get_cell(pos.offset(offset, offset)),
                    Quadrant::Northeast => ne.get_cell(pos.offset(-offset, offset)),
                    Quadrant::Southwest => sw.get_cell(pos.offset(offset, -offset)),
                    Quadrant::Southeast => se.get_cell(pos.offset(-offset, -offset)),
                }
            }
        }
    }

    pub fn set_cell_alive(&self, store: &mut Store, pos: Position) -> Rc<Node> {
        match &self.kind {
            NodeKind::Leaf { mut grid } => {
                let x_offset = (7 - pos.x) as usize;
                let y_offset = (pos.y + 8) as usize;
                grid = grid.replace(y_offset, grid.extract(y_offset) | (1 << x_offset));
                store.create_leaf(grid)
            }
            NodeKind::Interior { nw, ne, sw, se } => {
                // quarter side length
                let offset = 1 << (self.level - 2);

                match pos.quadrant() {
                    Quadrant::Northwest => {
                        let nw = nw.set_cell_alive(store, pos.offset(offset, offset));
                        store.create_interior(NodeTemplate {
                            nw,
                            ne: ne.clone(),
                            sw: sw.clone(),
                            se: se.clone(),
                        })
                    }
                    Quadrant::Northeast => {
                        let ne = ne.set_cell_alive(store, pos.offset(-offset, offset));
                        store.create_interior(NodeTemplate {
                            nw: nw.clone(),
                            ne,
                            sw: sw.clone(),
                            se: se.clone(),
                        })
                    }
                    Quadrant::Southwest => {
                        let sw = sw.set_cell_alive(store, pos.offset(offset, -offset));
                        store.create_interior(NodeTemplate {
                            nw: nw.clone(),
                            ne: ne.clone(),
                            sw,
                            se: se.clone(),
                        })
                    }
                    Quadrant::Southeast => {
                        let se = se.set_cell_alive(store, pos.offset(-offset, -offset));
                        store.create_interior(NodeTemplate {
                            nw: nw.clone(),
                            ne: ne.clone(),
                            sw: sw.clone(),
                            se,
                        })
                    }
                }
            }
        }
    }

    pub fn get_alive_cells(&self) -> Vec<Position> {
        match &self.kind {
            NodeKind::Leaf { grid } => {
                if grid.count_ones().wrapping_sum() == 0 {
                    return vec![];
                }

                let mut alive_coords = vec![];
                for y in -8..8 {
                    let y_offset = (y + 8) as usize;
                    let row = grid.extract(y_offset);
                    for x in -8..8 {
                        let x_offset = (7 - x) as usize;
                        if row & (1 << x_offset) > 0 {
                            alive_coords.push(Position { x, y });
                        }
                    }
                }
                alive_coords
            }
            NodeKind::Interior { nw, ne, sw, se } => {
                if self.population == 0 {
                    return vec![];
                }

                let mut alive_cells = vec![];

                // quarter side length
                let offset = 1 << (self.level - 2);

                alive_cells.extend(
                    nw.get_alive_cells()
                        .into_iter()
                        .map(|pos| pos.offset(-offset, -offset)),
                );
                alive_cells.extend(
                    ne.get_alive_cells()
                        .into_iter()
                        .map(|pos| pos.offset(offset, -offset)),
                );
                alive_cells.extend(
                    sw.get_alive_cells()
                        .into_iter()
                        .map(|pos| pos.offset(-offset, offset)),
                );
                alive_cells.extend(
                    se.get_alive_cells()
                        .into_iter()
                        .map(|pos| pos.offset(offset, offset)),
                );

                alive_cells
            }
        }
    }

    pub fn set_cells_alive(
        &self,
        store: &mut Store,
        coords: impl IntoIterator<Item = Position>,
    ) -> Rc<Node> {
        self.set_cells_alive_recursive(store, &mut coords.into_iter().collect::<Vec<_>>(), 0, 0)
    }

    fn set_cells_alive_recursive(
        &self,
        store: &mut Store,
        coords: &mut [Position],
        offset_x: i64,
        offset_y: i64,
    ) -> Rc<Node> {
        match &self.kind {
            NodeKind::Leaf { mut grid } => {
                for &mut pos in coords {
                    let x = (7 - (pos.x - offset_x)) as usize;
                    let y = ((pos.y - offset_y) + 8) as usize;
                    grid = grid.replace(y, grid.extract(y) | (1 << x));
                }
                store.create_leaf(grid)
            }
            NodeKind::Interior { nw, ne, sw, se } => {
                let (north, south) = partition_vert(coords, offset_y);
                let (northwest, northeast) = partition_horiz(north, offset_x);
                let (southwest, southeast) = partition_horiz(south, offset_x);

                // quarter side length
                let offset = 1 << (self.level - 2);

                let nw = if northwest.is_empty() {
                    store.create_empty(self.level - 1)
                } else {
                    nw.set_cells_alive_recursive(
                        store,
                        northwest,
                        offset_x - offset,
                        offset_y - offset,
                    )
                };
                let ne = if northeast.is_empty() {
                    store.create_empty(self.level - 1)
                } else {
                    ne.set_cells_alive_recursive(
                        store,
                        northeast,
                        offset_x + offset,
                        offset_y - offset,
                    )
                };
                let sw = if southwest.is_empty() {
                    store.create_empty(self.level - 1)
                } else {
                    sw.set_cells_alive_recursive(
                        store,
                        southwest,
                        offset_x - offset,
                        offset_y + offset,
                    )
                };
                let se = if southeast.is_empty() {
                    store.create_empty(self.level - 1)
                } else {
                    se.set_cells_alive_recursive(
                        store,
                        southeast,
                        offset_x + offset,
                        offset_y + offset,
                    )
                };

                store.create_interior(NodeTemplate { nw, ne, sw, se })
            }
        }
    }

    pub fn bounding_box(&self) -> Option<BoundingBox> {
        match &self.kind {
            NodeKind::Leaf { grid } => {
                if grid.count_ones().wrapping_sum() == 0 {
                    return None;
                }
                let mut min_x = i64::max_value();
                let mut min_y = i64::max_value();
                let mut max_x = i64::min_value();
                let mut max_y = i64::min_value();
                for y in -8..8 {
                    let y_offset = (y + 8) as usize;
                    let row = grid.extract(y_offset);
                    for x in -8..8 {
                        let x_offset = (7 - x) as usize;
                        if row & (1 << x_offset) > 0 {
                            min_x = min_x.min(x);
                            min_y = min_y.min(y);
                            max_x = max_x.max(x);
                            max_y = max_y.max(y);
                        }
                    }
                }
                Some(BoundingBox::new(
                    Position::new(min_x, min_y),
                    Position::new(max_x, max_y),
                ))
            }
            NodeKind::Interior { nw, ne, sw, se } => {
                if self.population == 0 {
                    return None;
                }

                // quarter side length
                let offset = 1 << (self.level - 2);

                let mut bounding_box = None::<BoundingBox>;

                if let Some(nw_bounding_box) = nw.bounding_box() {
                    let nw_bounding_box = nw_bounding_box.offset(-offset, -offset);
                    bounding_box = Some(nw_bounding_box);
                };

                if let Some(ne_bounding_box) = ne.bounding_box() {
                    let ne_bounding_box = ne_bounding_box.offset(offset, -offset);
                    bounding_box = if let Some(bbox) = bounding_box {
                        Some(bbox.combine(ne_bounding_box))
                    } else {
                        Some(ne_bounding_box)
                    }
                };

                if let Some(sw_bounding_box) = sw.bounding_box() {
                    let sw_bounding_box = sw_bounding_box.offset(-offset, offset);
                    bounding_box = if let Some(bbox) = bounding_box {
                        Some(bbox.combine(sw_bounding_box))
                    } else {
                        Some(sw_bounding_box)
                    }
                };

                if let Some(se_bounding_box) = se.bounding_box() {
                    let se_bounding_box = se_bounding_box.offset(offset, offset);
                    bounding_box = if let Some(bbox) = bounding_box {
                        Some(bbox.combine(se_bounding_box))
                    } else {
                        Some(se_bounding_box)
                    }
                };

                bounding_box
            }
        }
    }
}

fn partition_horiz(coords: &mut [Position], pivot: i64) -> (&mut [Position], &mut [Position]) {
    let mut next_index = 0;
    for i in 0..coords.len() {
        if coords[i].x < pivot {
            coords.swap(i, next_index);
            next_index += 1;
        }
    }
    coords.split_at_mut(next_index)
}

fn partition_vert(coords: &mut [Position], pivot: i64) -> (&mut [Position], &mut [Position]) {
    let mut next_index = 0;
    for i in 0..coords.len() {
        if coords[i].y < pivot {
            coords.swap(i, next_index);
            next_index += 1;
        }
    }
    coords.split_at_mut(next_index)
}
