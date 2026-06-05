import pytest
from typing import override, TYPE_CHECKING
from lib_ramis import PyState, CancelToken, GenericResult
from lib_ramis.binary import BinaryEvent, BinaryBFS
from lib_ramis.nary import create_nary_scheduler
from lib_ramis.traced import Trace, TracedBFS

# --- 1. Mock Subclasses for Testing ---

if TYPE_CHECKING:
    CounterStateBase = PyState[BinaryEvent]
else:
    CounterStateBase = PyState


class CounterState(CounterStateBase):
    """A custom state that tracks a counter based on Binary events."""

    def __init__(self, value: int):
        super().__init__()
        self.value: int = value

    @override
    def step(self, event: BinaryEvent) -> "CounterState":
        if event == BinaryEvent.Yes:
            return CounterState(self.value + 1)
        elif event == BinaryEvent.No:
            return CounterState(self.value - 1)


class SimpleToken(CancelToken):
    """A custom cancellation token."""

    def __init__(self):
        super().__init__()
        self._cancelled: bool = False

    @override
    def cancel(self) -> None:
        self._cancelled = True

    @override
    def is_cancelled(self) -> bool:
        return self._cancelled


# --- 2. The Tests ---


def test_generic_result_basics():
    """Test that the static methods and properties of GenericResult work."""
    res = GenericResult(10)
    assert res.raw_score() == 10
    assert not res.is_dead()

    dead_res = GenericResult(0)
    assert dead_res.raw_score() == 0
    assert dead_res.is_dead()


def test_binary_enum():
    """Test that the Binary enum exposes Yes/No variants properly."""
    assert BinaryEvent.Yes != BinaryEvent.No
    assert isinstance(BinaryEvent.Yes, BinaryEvent)


def test_custom_cancel_token():
    """Test that Rust can read the custom CancelToken subclass state."""
    token = SimpleToken()
    assert not token.is_cancelled()

    token.cancel()
    assert token.is_cancelled()


def test_bfs_scheduler_loop():
    """
    Integration test: drives the Rust BFS scheduler using custom Python subclasses.
    """
    initial_state = CounterState(0)
    token = SimpleToken()

    scheduler = BinaryBFS(initial_state)

    step1 = scheduler.next(token)
    assert step1 is not None

    state1 = step1.state()
    assert isinstance(state1, CounterState)

    assert state1.value in (1, -1)

    scheduler.put_result(step1, GenericResult(1))

    step2 = scheduler.next(token)
    assert step2 is not None

    state2: CounterState = step2.state()
    assert state2.value in (1, -1)
    assert state1.value != state2.value

    scheduler.notify_done()


def test_bfs_cancellation():
    """Ensures the scheduler respects the custom CancelToken."""
    initial_state = CounterState(10)
    token = SimpleToken()
    scheduler = BinaryBFS(initial_state)

    token.cancel()

    result = scheduler.next(token)
    assert result is not None

    scheduler.notify_done()
    next_step = scheduler.next(token)

    assert next_step is None


def test_traced_bfs_scheduler_loop():
    """
    Integration test: drives the TracedBFS scheduler.
    """
    token = SimpleToken()

    scheduler = TracedBFS()

    step1 = scheduler.next(token)
    assert step1 is not None

    trace1 = step1.path()
    assert isinstance(trace1, Trace)

    list1 = trace1.to_list()
    assert len(list1) == 1
    assert isinstance(list1[0], bool)

    scheduler.put_result(step1, GenericResult(1))

    step2 = scheduler.next(token)
    assert step2 is not None

    trace2 = step2.path()
    list2 = trace2.to_list()
    assert len(list2) == 1
    assert list1[0] != list2[0]

    scheduler.notify_done()


if TYPE_CHECKING:
    from lib_ramis.nary import NAryEvent

    NAryCounterStateBase = PyState[NAryEvent]
else:
    NAryCounterStateBase = PyState


class NAryCounterState(NAryCounterStateBase):
    """A custom state that tracks path decisions via the NAryEvent index."""

    def __init__(self, value: int):
        super().__init__()
        self.value: int = value

    @override
    def step(self, event: "NAryEvent") -> NAryCounterState:
        assert hasattr(event, "index")
        return NAryCounterState(self.value + event.index)


# --- Tests ---


def test_nary_scheduler_loop():
    """
    Integration test: drives the dynamic N-ary scheduler using the factory function.
    Ensures that compile-time array structures route properly via runtime branching factors.
    """
    branching_factor = 4
    initial_state = NAryCounterState(10)
    token = SimpleToken()

    scheduler = create_nary_scheduler(branching_factor, initial_state)

    assert scheduler.branching_factor == branching_factor

    step1 = scheduler.next(token)
    assert step1 is not None
    assert hasattr(step1, "state")

    state = step1.state
    assert isinstance(state, NAryCounterState)
    assert state.value < 10 + branching_factor

    scheduler.put_result(step1, GenericResult(42))

    step2 = scheduler.next(token)
    assert step2 is not None

    state2 = step2.state
    assert state2.value < 10 + 2 * branching_factor

    assert state2.value != state.value

    scheduler.put_result(step2, GenericResult(0))

    scheduler.notify_done()

    assert scheduler.next(token) is None


def test_nary_scheduler_invalid_branching_factor():
    """Ensures uncompiled branching factors raise a clean Python ValueError."""
    initial_state = NAryCounterState(0)

    with pytest.raises(ValueError, match="Unsupported branching factor: 999"):
        _ = create_nary_scheduler(999, initial_state)
