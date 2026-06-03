#![allow(clippy::type_complexity)]

use alloc::collections::VecDeque;
use core::{hash::Hash, iter::once, marker::PhantomData, ops::ControlFlow};

use ramis_core::{
    Algorithm,
    Cancellable,
    EventReplay,
    HasLevelStorage,
    OracleEvent,
    ScheduledStep,
    SelectionPolicy,
    StaticEvent,
    sync::{
        Arc,
        Mutex,
        Weak,
        atomic::{AtomicU64, Ordering},
    },
};
use smallvec::SmallVec;

use crate::StepScheduler;

#[derive(Hash, Clone, Default, Debug, PartialEq, Eq)]
pub struct RelativePath<E> {
    generation: u64,
    path: SmallVec<[E; 4]>,
}

impl<E> RelativePath<E> {
    pub fn new(generation: u64) -> Self {
        Self {
            generation,
            path: SmallVec::new(),
        }
    }

    pub fn new_from(generation: u64, path: impl Iterator<Item = E>) -> Self {
        Self {
            generation,
            path: path.collect(),
        }
    }
}

impl<E: StaticEvent + Clone + Eq> EventReplay for RelativePath<E> {
    type EventType = E;

    fn push(&mut self, event: Self::EventType) {
        self.path.push(event);
    }

    fn is_prefix_of(&self, other: &Self) -> bool {
        other.path.starts_with(&self.path)
    }

    fn extend_with_slice(&mut self, slice: &[Self::EventType]) {
        self.path.extend(slice.iter().cloned());
    }
}

pub enum Advanceable {
    Force,
    May,
}

pub struct TreeNode<E, C, S>
where
    C: Cancellable,
    E: HasLevelStorage,
{
    children: Mutex<E::LevelStorage<Option<Arc<TreeNode<E, C, S>>>>>,
    token: C,
    generation: u64,
    result: Mutex<Option<S>>,
    _phantom: PhantomData<E>,
}

impl<E, C, S> TreeNode<E, C, S>
where
    C: Cancellable,
    E: HasLevelStorage,
{
    pub fn new(token: C, generation: u64) -> Self {
        Self {
            children: Mutex::new(E::storage_from_fn(|_| None)),
            token,
            generation,
            result: Mutex::new(None),
            _phantom: PhantomData,
        }
    }

    pub fn walk_subtree<F, R>(zelf: Arc<Self>, f: &mut F) -> R
    where
        F: FnMut(Arc<Self>) -> ControlFlow<R, Arc<Self>>,
    {
        let mut root_node = zelf;
        loop {
            match f(root_node) {
                ControlFlow::Continue(node) => root_node = node,
                ControlFlow::Break(res) => return res,
            }
        }
    }

    pub fn may_advcance<F>(&self, f: F) -> bool
    where
        F: Fn(&S) -> Advanceable,
    {
        let mut may_advance = true;
        for c in self.children.lock().as_ref().iter() {
            match c {
                None => may_advance = false,
                Some(c) if c.result.lock().is_none() => may_advance = false,
                Some(c) if let Some(r) = c.result.lock().as_ref() => match f(r) {
                    Advanceable::May => {}
                    Advanceable::Force => return true,
                },
                _ => unreachable!(),
            }
        }

        may_advance
    }
}

impl<E, C, S> Drop for TreeNode<E, C, S>
where
    C: Cancellable,
    E: HasLevelStorage,
{
    fn drop(&mut self) {
        self.token.cancel();
    }
}

pub struct Tree<E, C, S>
where
    C: Cancellable,
    E: HasLevelStorage,
{
    children: Mutex<E::LevelStorage<Option<Arc<TreeNode<E, C, S>>>>>,
}

impl<E, C, S> Tree<E, C, S>
where
    C: Cancellable,
    E: HasLevelStorage,
{
    pub fn may_advcance<F>(&self, f: F) -> bool
    where
        F: Fn(&S) -> Advanceable,
    {
        let mut may_advance = true;
        for c in self.children.lock().as_ref().iter() {
            match c {
                None => may_advance = false,
                Some(c) if c.result.lock().is_none() => may_advance = false,
                Some(c) if let Some(r) = c.result.lock().as_ref() => match f(r) {
                    Advanceable::May => {}
                    Advanceable::Force => return true,
                },
                _ => unreachable!(),
            }
        }

        may_advance
    }
}

// TODO improve locking scheme in layout and usage

pub struct BFScheduler<T, E, C, S, P, A>
where
    C: Cancellable,
    E: HasLevelStorage,
{
    current_root: Mutex<T>,
    root_generation: AtomicU64,

    tasks: Mutex<Tree<E, C, S>>,
    frontier: Mutex<VecDeque<(RelativePath<E>, Weak<TreeNode<E, C, S>>)>>,

    _result: PhantomData<(S, P, A)>,
}

impl<T, E, C, S, P, A> Default for BFScheduler<T, E, C, S, P, A>
where
    C: Cancellable,
    T: Default,
    E: HasLevelStorage,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T, E, C, S, P, A> BFScheduler<T, E, C, S, P, A>
where
    C: Cancellable,
    E: HasLevelStorage,
{
    pub fn new(state: T) -> Self {
        Self {
            current_root: Mutex::new(state),
            root_generation: AtomicU64::new(0),
            tasks: Mutex::new(Tree {
                children: Mutex::new(E::storage_from_fn(|_| None)),
            }),
            frontier: Mutex::new(VecDeque::new()),
            _result: PhantomData,
        }
    }
}

impl<T, E, C, S, P, A> StepScheduler<T, C> for BFScheduler<T, E, C, S, P, A>
where
    C: Cancellable + Clone,
    T: Clone,
    E: StaticEvent + Clone + Eq,
    P: SelectionPolicy<OracleEvent = S>,
    A: Algorithm<T, E>,
    S: OracleEvent + Clone,
{
    type ItemMeta = Weak<TreeNode<E, C, S>>;
    type StateInterpretation = S;

    fn next(&self, token: C) -> Result<ScheduledStep<T, Self::ItemMeta>, C> {
        // TODO we could recheck generation in the loop and restart if it does not match anymore
        let (mut state, path_to_apply, weak) = 'get: {
            let mut frontier = self.frontier.lock();
            let tasks = self.tasks.lock();

            let root_guard = self.current_root.lock();
            let current_gen = self.root_generation.load(Ordering::Acquire);
            if current_gen == u64::MAX {
                return Err(token);
            }
            let root = root_guard.clone();
            drop(root_guard);

            let mut root_children = tasks.children.lock();

            for (variant, child) in E::VARIANTS
                .as_ref()
                .iter()
                .cloned()
                .zip(root_children.as_mut().iter_mut())
            {
                if child.is_none() {
                    let node = Arc::new(TreeNode::new(token.clone(), current_gen + 1));
                    let weak = Arc::downgrade(&node);
                    *child = Some(node);

                    let rel_path = RelativePath::new_from(current_gen, once(variant.clone()));
                    frontier.push_back((rel_path.clone(), weak.clone()));

                    break 'get (root.clone(), rel_path.path, weak);
                }
            }

            while let Some((mut rel_path, parent_node)) = frontier.pop_front() {
                let Some(parent_node_strong) = parent_node.upgrade() else {
                    continue;
                };
                if parent_node_strong
                    .result
                    .lock()
                    .as_ref()
                    .is_some_and(P::may_reject)
                {
                    continue;
                }

                let mut children = parent_node_strong.children.lock();
                for (variant, child) in E::VARIANTS
                    .as_ref()
                    .iter()
                    .cloned()
                    .zip(children.as_mut().iter_mut())
                {
                    if child.is_none() {
                        let current_gen = self.root_generation.load(Ordering::Acquire);
                        if rel_path.generation < current_gen {
                            rel_path
                                .path
                                .drain(..(current_gen - rel_path.generation) as usize);
                            rel_path.generation = current_gen;
                        }

                        let mut child_path = RelativePath::new_from(
                            rel_path.generation,
                            rel_path.path.iter().cloned(),
                        );
                        child_path.push(variant);

                        let node = Arc::new(TreeNode::new(
                            token.clone(),
                            parent_node_strong.generation + 1,
                        ));
                        let weak = Arc::downgrade(&node);
                        *child = Some(node);

                        frontier.push_front((rel_path, parent_node));
                        frontier.push_back((child_path.clone(), weak.clone()));

                        break 'get (root.clone(), child_path.path, weak);
                    }
                }
            }
            return Err(token);
        };

        for ev in path_to_apply {
            if let Err(_e) = A::step(&mut state, ev) {
                // since we put the node into th tree already, we should try to mark it as dead
                // we can also immediately reap all of its children, as we did just put this node back into the queue
                // This does NOT ensure that no other thread runs a child/puts one back into the queue.
                // All remaining children will be reaped on the next root advance.
                if let Some(strong) = weak.upgrade() {
                    let mut r_lock = strong.result.lock();
                    debug_assert!(r_lock.is_none(), "someone else evaluated our node??");
                    *r_lock = Some(<S as OracleEvent>::DEAD.clone());
                    drop(r_lock);
                    let mut children = strong.children.lock();
                    children.as_mut().iter_mut().for_each(|child| *child = None);
                }
                return Err(token);
            }
        }

        Ok(ScheduledStep::new(state, weak))
    }

    fn put_result(&self, path: ScheduledStep<T, Self::ItemMeta>, event: Self::StateInterpretation) {
        let advancement_data = {
            let tasks = self.tasks.lock();

            let Some(node) = path.meta().upgrade() else {
                // already cancelled
                return;
            };

            let current_generation = self.root_generation.load(Ordering::Acquire);

            let item_gen = node.generation;

            *node.result.lock() = Some(event);

            if node.result.lock().as_ref().is_some_and(P::may_reject) {
                // reap all children
                // Note that it is possible for a child to be correct. Since we do not search for global optimum, this does not matter. Any path to q-minimality is fine.

                // retain all branches that are
                // a) not a subtree of us

                // then drop all of our children.
                // This will invalidate Weak references in frontier and tasks and ensure no further exploration (because tasks is locked right now, no radce exists).
                // This is simply here because we may not be the last of our siblings to be done. in this case we can already remove our subtree
                node.children
                    .lock()
                    .as_mut()
                    .iter_mut()
                    .for_each(|c| *c = None);
            }

            // reap all non-children and set as root if our generation == current_generation
            // We check if all siblings of node are done. We know that nodes siblings are roots chidlren, since our generation is root_gen + 1

            if item_gen == current_generation + 1
                && tasks.may_advcance(|r| {
                    if P::may_accept(r) {
                        Advanceable::Force
                    } else {
                        Advanceable::May
                    }
                })
            {
                // we are now the new root.
                // walk our subtree until we find no new suitable root
                // finally update the tree root to drop all tasks not on our subtree and extend root path

                let find_best = |children: &[Option<Arc<TreeNode<E, C, S>>>]| -> Option<(E, Arc<TreeNode<E, C, S>>)> {
                    let mut best = None;
                    for (variant, child) in E::VARIANTS.as_ref().iter().zip(children.iter()) {
                        let Some(child) = child else { continue; };
                        let res_lock = child.result.lock();
                        let Some(res) = res_lock.as_ref() else { continue; };

                        if P::may_reject(res) {
                            continue;
                        }

                        if P::may_accept(res) {
                            return Some((variant.clone(), child.clone()));
                        }

                        if best.as_ref().is_none_or(|(_, best): &(E, Arc<TreeNode<E, C, S>>)| {
                            P::compare(best.result.lock().as_ref().unwrap(), res) ==  core::cmp::Ordering::Less
                        }) {
                            best = Some((variant.clone(), child.clone()));
                        }
                    }
                    best
                };

                let Some((variant, mut lowest_node)) = find_best(tasks.children.lock().as_ref())
                else {
                    return;
                };

                let mut acc = alloc::vec![variant];

                TreeNode::walk_subtree(lowest_node.clone(), &mut |node| {
                    if !node.may_advcance(|r| {
                        if P::may_accept(r) {
                            Advanceable::Force
                        } else {
                            Advanceable::May
                        }
                    }) {
                        return ControlFlow::Break(());
                    }
                    if let Some((variant, next_node)) = find_best(node.children.lock().as_ref()) {
                        acc.push(variant);
                        lowest_node = next_node.clone();
                        ControlFlow::Continue(next_node)
                    } else {
                        ControlFlow::Break(())
                    }
                });

                Some((acc, lowest_node, current_generation))
            } else {
                None
            }
        };

        if let Some((acc, lowest_node, current_generation)) = advancement_data {
            // we clone first, drop guard, step and finally lock again in order to allow concurrent workers to continue running
            // root needs to be locked by next and put_result().
            // If we keep it locked during a potentially long step(), we could serialize the scheduler
            // HOWEVER this means that
            // a) a lot of work is potentially wasted in concurretn workers (as we will advance thge root soon, dropping many paths)
            // b) more clones than necessary are performed here. If State is very heavy, this could be very bad
            //
            // TODO we should maybe allow the user to make this choice, or do some benchmarking
            let mut new_root = {
                let guard = self.current_root.lock();
                guard.clone()
            };

            for ev in acc {
                A::step(&mut new_root, ev).expect("Algorihtm erred in put_result. This should not happen, as the exact same state was previously evaluated in next.");
            }

            let mut tasks = self.tasks.lock();
            let mut root_guard = self.current_root.lock();

            if self
                .root_generation
                .compare_exchange(
                    current_generation,
                    lowest_node.generation,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
            {
                // we won any possible race to the root update and can safely update the root
                *root_guard = new_root;

                let lowest_children = lowest_node.children.lock();
                *tasks = Tree {
                    children: Mutex::new(<E as HasLevelStorage>::storage_from_fn(|idx| {
                        lowest_children.as_ref()[idx].clone()
                    })),
                };
            }
        }
    }

    fn notify_done(&self) {
        self.root_generation.store(u64::MAX, Ordering::Release);
        self.frontier.lock().clear();
        self.tasks
            .lock()
            .children
            .lock()
            .as_mut()
            .iter_mut()
            .for_each(|ele| *ele = None);
    }

    fn is_cancelled(&self, item: &ScheduledStep<T, Self::ItemMeta>) -> bool {
        item.meta()
            .upgrade()
            .is_none_or(|item| item.token.is_cancelled())
    }
}
