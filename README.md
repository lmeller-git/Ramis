[![Codecov](https://codecov.io/github/lmeller-git/Ramis/coverage.svg?branch=main)](https://codecov.io/gh/lmeller-git/Ramis)
![CI Test](https://github.com/lmeller-git/Ramis/actions/workflows/test.yml/badge.svg?branch=main)
![Safety Test](https://github.com/lmeller-git/Ramis/actions/workflows/safety.yml/badge.svg?branch=main)
![no_std Test](https://github.com/lmeller-git/Ramis/actions/workflows/nostd.yml/badge.svg?branch=main)

# Ramis

> Concurrent tree search, branch by branch.

A framework and implementation for concurrently running a class of tree search algorithms.
Ramis decouples the specific algorithm from its concurrent state exploration logic, allowing
easy and efficient implementation of concurrent algorithms.

---

## Supported Algorithms

`Ramis` can run any algorithm expressible as an **oracle-guided tree search** satisfying following
constraints:

1. **Tree structure** — the search space is an (implicit) tree where children of a node
   represent one step of refinement (for example a smaller test case)
2. **Pure oracle** — acceptance depends only on the candidate's value, not on evaluation
   order or concurrent context. In other words the state at node $N$ should depend entirely on its trace.
3. **Enumerable neighbourhood** — For a node $N$ it must be decidable which child of it is best. This means that all children need to be constructable, or a forced accept needs to be injected

---

## Guarantees

*All following guarantees depend on the generic algorithm allowing such a guarantee and being sound in that regard.*

**1-minimality.** The returned state has no single-step improvement: the oracle rejects every
immediate child of the result. `Ramis` achieves this by evaluating *every* child of a node
concurrently before advancing.

**N concurrent workers.** $N$ algorithm workers and oracle workers may be run concurrently. The scheduler ensures utilization of all workers.

**Soundness.** Every state accepted and retained by the scheduler satisfies the oracle.

---

## Usage

Describe your algorithm in terms of the trait `ramis::traits::SearchDomain`, construct a `ramis::schedule::BFS` with this spec and run it on your problem space.

Examples may be found in `examples/`.

---

## Feature Flags

- `std`: Enables `std` support

- `default`: `std`

For `no_std` targets `alloc` is required.

---

## Bindings

- Python bindings are available via `lib_ramis`
