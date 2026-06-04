from typing import override
from lib_ramis import PyState, CancelToken, GenericResult
from lib_ramis.binary import Binary, BinaryBFS

# --- 1. Mock Subclasses for Testing ---


class CounterState(PyState):
    """A custom state that tracks a counter based on Binary events."""

    def __init__(self, value: int):
        super().__init__()
        self.value: int = value

    @override
    def step(self, event: Binary) -> "CounterState":
        if event == Binary.Yes:
            return CounterState(self.value + 1)
        elif event == Binary.No:
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
    assert Binary.Yes != Binary.No
    assert isinstance(Binary.Yes, Binary)


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

    state2: CounterState  = step2.state()
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
    next = scheduler.next(token)

    assert next is None
