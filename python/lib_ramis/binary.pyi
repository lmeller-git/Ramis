import enum
from lib_ramis import CancelToken, PyState, GenericResult

class Binary(enum.Enum):
    No = 0
    Yes = 1

class BinaryBFSStep:
    def state(self) -> PyState: ...

class BinaryBFS:
    def __init__(self, state: PyState) -> None: ...

    def next(self, cancel_token: CancelToken) -> BinaryBFSStep | None: ...

    def put_result(self, step: BinaryBFSStep, result: GenericResult) -> None: ...

    def notify_done(self) -> None: ...

    def is_cancelled(self, item: BinaryBFSStep) -> bool: ...
