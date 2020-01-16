from slonik import rust
from slonik._native import ffi
from slonik._native import lib

from .row import _Row
from .row import Row
from .row import deserialize_item


class _Result(rust.RustObject):
    def next_row(self):
        row = self._methodcall(lib.next_row)
        if row == ffi.NULL:
            return
        return _Row._from_objptr(row)

    def close(self):
        self._methodcall(lib.result_close)


class Result:
    def __init__(self, _result):
        self._result = _result

    def __iter__(self):
        return self

    def __next__(self):
        if self._result is None:
            raise StopIteration

        _row = self._result.next_row()
        if _row is None:
            raise StopIteration

        with Row(_row) as row:
            return tuple(row)

    def close(self):
        if self._result is not None:
            self._result.close()
        self._result = None

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        self.close()


class _EagerResult(rust.RustObject):
    def items(self):
        return self._methodcall(lib.eager_result_get_items)

    def close(self):
        self._methodcall(lib.eager_result_close)


class EagerResult:
    def __init__(self, _result):
        self._result = _result

    def close(self):
        if self._result is not None:
            self._result.close()
        self._result = None

    def rows(self):
        items = self._result.items()

        row_count = items.len
        col_count = items.stride

        # the 2D query results are stored as a row-major 1D array, with:
        # - `row_count` rows
        # - `col_count` items per row
        for row in range(row_count):
            row_start = row * col_count
            row_end = row_start + col_count
            row_items = items.ptr[row_start:row_end]

            yield (*[
                deserialize_item(
                    rust.buff_to_bytes(item.type_name), rust.buff_to_bytes(item.value))
                for item in row_items
            ],)

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        self.close()
