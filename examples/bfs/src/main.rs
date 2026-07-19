use std::{
    cmp::Ordering,
    collections::{HashSet, VecDeque},
    fmt::Display,
    num::NonZero,
    sync::Arc,
    time::Instant,
};

use dashmap::DashSet;
use ramis::{
    components::GenericOracleEvent,
    schedule::{BFS, BranchDirective},
    traits::{
        Algorithm,
        Cancellable,
        HasLevelStorage,
        SearchDomain,
        SelectionPolicy,
        StaticEvent,
        StepScheduler,
    },
};
use tokio_util::sync::CancellationToken;

// We have a n x m grid, with some cells being blocked.
// On this grid L white horses and K black horses are placed in some initial positions.
// Our objective is to swap the positions of the black and white horses.
// Horses may move according to standard chess rules.

const FREE: u8 = 0;
const W: u8 = 1;
const B: u8 = 2;
const BLOCK: u8 = 3;

const G_SIZE_X: usize = 4;
const G_SIZE_Y: usize = 4;

const TOTAL_PIECES: usize = 4;

const ARTIFICIAL_DELAY: u64 = 0;

const BLOCKED_LUT: [[bool; G_SIZE_X]; G_SIZE_Y] = [
    [true, false, true, true],
    [true, false, false, true],
    [true, false, false, false],
    [false, false, false, false],
];

const DIRECTIONS: [(i8, i8); 8] = [
    (-1, -2),
    (-1, 2),
    (1, -2),
    (1, 2),
    (-2, -1),
    (-2, 1),
    (2, -1),
    (2, 1),
];

fn shift_pos(pos: (usize, usize), dx: i8, dy: i8) -> Option<(usize, usize)> {
    let x = pos.0 as i8 + dx;
    let y = pos.1 as i8 + dy;
    if x >= 0 && x < G_SIZE_X as i8 && y >= 0 && y < G_SIZE_Y as i8 {
        Some((x as usize, y as usize))
    } else {
        None
    }
}

#[allow(clippy::type_complexity)]
fn init_puzzle() -> (Grid, Grid, [((usize, usize), u8); TOTAL_PIECES]) {
    let mut grid = Grid::new();
    let mut target_grid = Grid::new();

    let current_positions = [((0, 1), W), ((2, 2), W), ((3, 0), B), ((3, 2), B)];
    let win_condition = [((0, 1), B), ((2, 2), B), ((3, 0), W), ((3, 2), W)];

    grid.set_state(&current_positions);
    target_grid.set_state(&win_condition);

    (grid, target_grid, current_positions)
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct Grid {
    map: [[u8; G_SIZE_X]; G_SIZE_Y],
    stack: Vec<((usize, usize), (usize, usize))>,
}

impl Grid {
    fn new() -> Self {
        let mut grid = [[FREE; G_SIZE_X]; G_SIZE_Y];
        for (i, row) in BLOCKED_LUT.iter().enumerate() {
            for (j, &col) in row.iter().enumerate() {
                if col {
                    grid[i][j] = BLOCK;
                }
            }
        }
        Self {
            map: grid,
            stack: Vec::new(),
        }
    }

    fn set_state(&mut self, state: &[((usize, usize), u8); TOTAL_PIECES]) {
        for (pos, v) in state {
            self.map[pos.0][pos.1] = *v
        }
    }

    fn try_apply(
        &mut self,
        potential_move_from: (usize, usize),
        potential_move_to: (usize, usize),
    ) -> bool {
        if self.map[potential_move_to.0][potential_move_to.1] != FREE {
            return false;
        }

        self.map[potential_move_to.0][potential_move_to.1] =
            self.map[potential_move_from.0][potential_move_from.1];
        self.map[potential_move_from.0][potential_move_from.1] = FREE;

        self.stack.push((potential_move_from, potential_move_to));

        true
    }

    fn pop(&mut self) {
        if let Some((potential_move_to, potential_move_from)) = self.stack.pop() {
            self.map[potential_move_to.0][potential_move_to.1] =
                self.map[potential_move_from.0][potential_move_from.1];
            self.map[potential_move_from.0][potential_move_from.1] = FREE;
        }
    }

    fn is_finished(&self, target: &Self) -> bool {
        std::thread::sleep(std::time::Duration::from_millis(ARTIFICIAL_DELAY));
        self.map == target.map
    }
}

impl Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Grid")?;
        for row in self.map {
            write!(f, "|")?;
            for col in row {
                match col {
                    BLOCK => write!(f, "  ")?,
                    FREE => write!(f, "_")?,
                    W => write!(f, "W")?,
                    B => write!(f, "B")?,
                    _ => panic!(),
                }
                write!(f, "|")?;
            }
            writeln!(f)?
        }

        for (from, to) in &self.stack {
            writeln!(f, "{:?} -> {:?}", from, to)?;
        }

        Ok(())
    }
}

// 'Naive' sequential bfs for the solution to the riddle

fn bfs() {
    let (grid, target_grid, current_positions) = init_puzzle();

    let mut queue = VecDeque::new();
    let mut seen = HashSet::new();

    seen.insert(grid.map);
    queue.push_back((current_positions, grid.clone()));

    while let Some((positions, mut grid)) = queue.pop_front() {
        for (i, (from, _)) in positions.iter().enumerate() {
            for to in move_iter(*from) {
                if grid.try_apply(*from, to) {
                    let mut next = positions;
                    next[i].0 = to;
                    if grid.is_finished(&target_grid) {
                        println!("{}", grid);
                        return;
                    }
                    if !seen.insert(grid.map) {
                        grid.pop();
                        continue;
                    }

                    queue.push_back((next, grid.clone()));
                    grid.pop();
                }
            }
        }
    }
}

fn move_iter(position: (usize, usize)) -> impl Iterator<Item = (usize, usize)> {
    DIRECTIONS
        .iter()
        .filter_map(move |&(dx, dy)| shift_pos(position, dx, dy))
}

// The above 'naive' solution can be reformulated as a ramis parallel search and then run with a thread pool or an async runtime.
// This leads to massive performance boosts, if the oracle (`Grid::is_finished`) is slow, which we simulate by adding a `std::thread::sleep`.

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct StateMachine {
    grid: Grid,
    position_lut: [((usize, usize), u8); TOTAL_PIECES],
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct PuzzleMove {
    piece: u8,
    dx: i8,
    dy: i8,
}

impl HasLevelStorage for PuzzleMove {
    type LevelStorage<T> = [T; 8 * TOTAL_PIECES];

    fn storage_from_fn<T, F: FnMut(usize) -> T>(f: F) -> [T; 8 * TOTAL_PIECES] {
        core::array::from_fn(f)
    }
}

impl StaticEvent for PuzzleMove {
    const VARIANTS: &'static [PuzzleMove; 8 * TOTAL_PIECES] = &{
        let mut out = [PuzzleMove {
            piece: 0,
            dx: 0,
            dy: 0,
        }; 8 * TOTAL_PIECES];
        let mut i = 0;
        while i < 8 * TOTAL_PIECES {
            out[i] = PuzzleMove {
                piece: (i / 8) as u8,
                dx: DIRECTIONS[i % 8].0,
                dy: DIRECTIONS[i % 8].1,
            };
            i += 1;
        }
        out
    };
}
struct StateDriver;

impl Algorithm<StateMachine, PuzzleMove> for StateDriver {
    type Error = ();

    fn step(state: &mut StateMachine, event: PuzzleMove) -> Result<(), Self::Error> {
        let position = &mut state.position_lut[event.piece as usize].0;

        if let Some(next_pos) = shift_pos(*position, event.dx, event.dy)
            && state.grid.try_apply(*position, next_pos)
        {
            *position = next_pos;
            return Ok(());
        }

        Err(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct RiddleCancelToken {
    token: CancellationToken,
}

impl RiddleCancelToken {
    pub fn new() -> Self {
        Self {
            token: CancellationToken::new(),
        }
    }

    pub fn token(&self) -> &CancellationToken {
        &self.token
    }
}

impl Cancellable for RiddleCancelToken {
    fn cancel(&self) {
        self.token.cancel();
    }

    fn is_cancelled(&self) -> bool {
        self.token.is_cancelled()
    }
}

// Note that we return `BranchDirective::Hold` in the usual case, as we must explore all branches to find the solution and do not have "correct intermediate steps".
pub struct RiddlePolicy;

impl SelectionPolicy for RiddlePolicy {
    type OracleEvent = GenericOracleEvent;

    fn compare(_a: &Self::OracleEvent, _b: &Self::OracleEvent) -> Ordering {
        Ordering::Equal
    }

    fn branch_directive(s: &Self::OracleEvent) -> BranchDirective {
        match s {
            GenericOracleEvent::Dead => BranchDirective::Prune,
            GenericOracleEvent::Alive(_) => BranchDirective::Hold,
            GenericOracleEvent::Accept => BranchDirective::Force,
        }
    }
}

struct Domain;

impl SearchDomain for Domain {
    type Algorithm = StateDriver;
    type Event = PuzzleMove;
    type Policy = RiddlePolicy;
    type State = StateMachine;
}

// Parrallel bfs using a thread pool

fn par_bfs() {
    let (grid, target_grid, current_positions) = init_puzzle();

    let seen = Arc::new(DashSet::new());
    seen.insert(grid.map);

    let state_machine = StateMachine {
        grid,
        position_lut: current_positions,
    };

    let scheduler: Arc<BFS<Domain, RiddleCancelToken>> = BFS::new(state_machine).into();
    let target_grid = Arc::new(target_grid);

    let num_workers = std::thread::available_parallelism()
        .unwrap_or(NonZero::new(8).unwrap())
        .into();

    let mut workers = vec![];

    for _ in 0..num_workers {
        let scheduler = Arc::clone(&scheduler);
        let seen = Arc::clone(&seen);
        let target = Arc::clone(&target_grid);

        let handle = std::thread::spawn(move || {
            loop {
                let token = RiddleCancelToken::default();
                let next_state = match scheduler.next(token) {
                    Ok(state) => state,
                    Err(ramis::schedule::StepError::Terminated(_)) => break,
                    Err(ramis::schedule::StepError::Busy(_)) => {
                        std::thread::yield_now();
                        continue;
                    }
                    Err(_) => continue,
                };

                if next_state.path().grid.is_finished(&target) {
                    println!("{}", next_state.path().grid);
                    scheduler.notify_done();
                    break;
                }

                if !seen.insert(next_state.path().grid.map) {
                    scheduler.put_result(next_state, GenericOracleEvent::Dead);
                } else {
                    scheduler.put_result(next_state, GenericOracleEvent::Alive(0));
                }
            }
        });
        workers.push(handle);
    }

    for worker in workers {
        worker.join().unwrap();
    }
}

// Parallel bfs using tokio runtime

async fn run_worker(
    scheduler: Arc<BFS<Domain, RiddleCancelToken>>,
    seen: Arc<DashSet<[[u8; TOTAL_PIECES]; TOTAL_PIECES]>>,
    target_grid: &Grid,
) {
    loop {
        let token = RiddleCancelToken::new();

        let path = match scheduler.next(token.clone()) {
            Ok(path) => path,
            Err(ramis::schedule::StepError::Terminated(_)) => return,
            Err(ramis::schedule::StepError::Busy(_)) => {
                tokio::task::yield_now().await;
                continue;
            }
            Err(_) => continue,
        };

        if !seen.insert(path.path().grid.map) {
            scheduler.put_result(path, GenericOracleEvent::Dead);
            continue;
        }

        let (tx, rx) = tokio::sync::oneshot::channel();
        let current_grid = path.path().grid.clone();
        let target_clone = target_grid.clone();

        tokio::task::spawn_blocking(move || {
            let finished = current_grid.is_finished(&target_clone);
            let _ = tx.send(finished);
        });

        tokio::select! {
            _ = token.token().cancelled() => {}
            Ok(is_goal_state) = rx => {
                if is_goal_state {
                    println!("{}", path.path().grid);
                    scheduler.notify_done();
                    return;
                } else {
                    scheduler.put_result(path, GenericOracleEvent::Alive(0));
                }
            }
        }
    }
}

async fn async_par() {
    let (grid, target_grid, current_positions) = init_puzzle();

    let seen = Arc::new(DashSet::new());
    seen.insert(grid.map);

    let state_machine = StateMachine {
        grid,
        position_lut: current_positions,
    };

    let scheduler = Arc::new(BFS::<Domain, RiddleCancelToken>::new(state_machine));
    let target_grid = Arc::new(target_grid);

    let num_workers = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(8)
        * 2;

    let mut handles = Vec::new();

    for _ in 0..num_workers {
        let sched_clone = Arc::clone(&scheduler);
        let seen_clone = Arc::clone(&seen);
        let target = Arc::clone(&target_grid);

        handles.push(tokio::spawn(async move {
            run_worker(sched_clone, seen_clone, &target).await;
        }));
    }

    for handle in handles {
        let _ = handle.await;
    }
}

#[tokio::main]
async fn main() {
    let now = Instant::now();
    par_bfs();
    println!("par used {:?} time", Instant::now() - now);

    let now = Instant::now();
    async_par().await;
    println!("async par used {:?} time", Instant::now() - now);

    let now = Instant::now();
    bfs();
    println!("seq used {:?} time", Instant::now() - now);
}
