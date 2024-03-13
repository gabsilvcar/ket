"""Base quantum programming classes.

This module provides base classes for handling quantum programming in the Ket library. It includes
for handle and store quantum states, measurements.
"""

from __future__ import annotations

# SPDX-FileCopyrightText: 2020 Evandro Chagas Ribeiro da Rosa <evandro@quantuloop.com>
# SPDX-FileCopyrightText: 2020 Rafael de Santiago <r.santiago@ufsc.br>
#
# SPDX-License-Identifier: Apache-2.0

from cmath import sqrt
from collections import defaultdict
from ctypes import c_size_t, c_uint8
from json import loads
from random import Random
from typing import Literal, Optional, Any

from .clib.libket import Process as LibketProcess
from .clib.kbw import get_simulator

__all__ = [
    "Process",
    "Quant",
    "Measurement",
    "Samples",
    "QuantumState",
    "set_default_process_configuration",
]

DEFAULT_PROCESS_CONFIGURATION = {
    "configuration": None,
    "num_qubits": None,
    "simulator": None,
    "execution": None,
    "force": False,
}


def set_default_process_configuration(
    configuration=None,
    num_qubits: Optional[int] = None,
    simulator: Optional[Literal["sparse", "dense"]] = None,
    execution: Optional[Literal["live", "batch"]] = None,
    force_configuration: bool = False,
):
    """Set default process configurations.

    Configures default parameters for quantum processes using the specified options.

    Args:
        configuration: Configuration definition for third-party quantum execution. Defaults to None.
        num_qubits: Number of qubits for the KBW simulator.Defaults to None.
        simulator: Simulation mode for the KBW simulator. Defaults to None.
        execution: Execution mode for the KBW simulator. Defaults to None.
        force_configuration: If set to True, the parameters defined in the
            :class:`~ket.base.Process` constructor will overwrite those that are not None. Defaults
            to False.
    """

    global DEFAULT_PROCESS_CONFIGURATION  # pylint: disable=global-statement

    new_configuration = {
        "configuration": configuration,
        "num_qubits": num_qubits,
        "simulator": simulator,
        "execution": execution,
        "force": force_configuration,
    }

    DEFAULT_PROCESS_CONFIGURATION = new_configuration


class Process(LibketProcess):
    """Quantum program process.

    A :class:`~ket.base.Process` in Ket handles quantum circuit preparation and execution, serving
    as a direct interface to the Rust library. Its primary usage in quantum programming is to
    allocate qubits using the :meth:`~ket.base.Process.alloc` method.

    Example:

        .. code-block:: python

            from ket import Process

            p = Process()
            qubits = p.alloc(10) # Allocate 10 qubits

    By default, quantum execution is performed by the KBW simulator using sparse mode with 32
    qubits. The KBW simulator in sparse mode handles qubits in a representation equivalent to a
    sparse matrix. This simulator's execution time is related to the amount of superposition in the
    quantum execution, making it suitable as a default when the number of qubits is unknown or the
    quantum state can be represented by a sparse matrix, such as in a GHZ state. The dense simulator
    mode has exponential time complexity with the number of qubits. While it can better explore CPU
    parallelism, the number of qubits must be carefully set. The default number of qubits for the
    dense simulator is 12. The choice of simulator mode depends on the quantum algorithm, as each
    mode has its pros and cons.

    Another parameter for quantum execution on the KBW simulator is between "live" and "batch"
    execution. This configuration determines when quantum instructions will be executed. If set to
    "live", quantum instructions execute immediately after the call. If set to "batch", quantum
    instructions execute only at the end of the process. The default configuration is "live",
    suitable for quantum simulation where the non-clone theorem of quantum mechanics does not need
    to be respected. Batch execution is closer to what is expected from a quantum computer and is
    recommended when preparing quantum code to execute on a QPU.

    Example:

        Batch execution:

        .. code-block:: python

            from ket import *

            p = Process(execution="batch")
            a, b = p.alloc(2)

            CNOT(H(a), b) # Bell state preparation

            d = dump(a + b)

            p.execute()
            # The value of `d` will only be available after executing the process

            print(d.show())

            CNOT(a, b)  # This instruction will raise an error since the process
                        # has already executed.

        Live execution:

        .. code-block:: python

            from ket import *

            p = Process(execution="batch")
            a, b = p.alloc(2)

            CNOT(H(a), b) # Bell state preparation

            # The value of the dump is available right after
            print(dump(a + b).show())

            CNOT(a, b)  # This instruction can execute normally
            H(a)

            print(dump(a + b).show())


    Args:
        configuration: Configuration definition for third-party quantum execution. Defaults to None.
        num_qubits: Number of qubits for the KBW simulator. If None and ``simulator=="sparse"``,
            defaults to 32; otherwise, defaults to 12.
        simulator: Simulation mode for the KBW simulator. If None, defaults to ``"sparse"``.
        execution: Execution mode for the KBW simulator. If None, defaults to ``"live"``.
    """

    def __init__(
        self,
        configuration=None,
        num_qubits: Optional[int] = None,
        simulator: Optional[Literal["sparse", "dense"]] = None,
        execution: Optional[Literal["live", "batch"]] = None,
    ):
        if DEFAULT_PROCESS_CONFIGURATION["force"] or all(
            map(lambda a: a is None, [configuration, num_qubits, simulator, execution])
        ):
            configuration = (
                DEFAULT_PROCESS_CONFIGURATION["configuration"]
                if DEFAULT_PROCESS_CONFIGURATION["configuration"] is not None
                else configuration
            )
            num_qubits = (
                DEFAULT_PROCESS_CONFIGURATION["num_qubits"]
                if DEFAULT_PROCESS_CONFIGURATION["num_qubits"] is not None
                else num_qubits
            )
            simulator = (
                DEFAULT_PROCESS_CONFIGURATION["simulator"]
                if DEFAULT_PROCESS_CONFIGURATION["simulator"] is not None
                else simulator
            )
            execution = (
                DEFAULT_PROCESS_CONFIGURATION["execution"]
                if DEFAULT_PROCESS_CONFIGURATION["execution"] is not None
                else execution
            )

        if configuration is not None and any(
            map(lambda a: a is not None, [num_qubits, simulator, execution])
        ):
            raise ValueError(
                "Cannot specify num_qubits, simulator or execution if configuration is provided"
            )

        if configuration is not None:
            super().__init__(configuration)
        else:
            simulator = "sparse" if simulator is None else simulator
            num_qubits = (
                (32 if simulator == "sparse" else 12)
                if num_qubits is None
                else num_qubits
            )
            super().__init__(
                get_simulator(
                    num_qubits=num_qubits,
                    simulator=simulator,
                    execution="live" if execution is None else execution,
                )
            )

        self._metadata_buffer_size = 512
        self._metadata_buffer = (c_uint8 * self._metadata_buffer_size)()
        self._instructions_buffer_size = 2048
        self._instructions_buffer = (c_uint8 * self._instructions_buffer_size)()

    def alloc(self, num_qubits: int = 1) -> Quant:
        """Allocate a specified number of qubits and return a :class:`~ket.base.Quant` object.

        Args:
            num_qubits: The number of qubits to allocate. Defaults to 1.

        Returns:
            A list like object representing the allocated qubits.


        Example:
            >>> from ket import Process
            >>> p = Process()
            >>> qubits = p.alloc(3)
            >>> print(qubits)
            <Ket 'Quant' [0, 1, 2] pid=0x...>

        The :meth:`~ket.base.Process.alloc` method allocates the specified number of qubits and
        returns a :class:`~ket.base.Quant` object. Each qubit is assigned a unique index, and the
        resulting :class:`~ket.base.Quant` object encapsulates the allocated qubits along with a
        reference to the parent :class:`~ket.base.Process` object.
        """

        if num_qubits < 1:
            raise ValueError("Cannot allocate less than 1 qubit")

        qubits_index = [self.allocate_qubit().value for _ in range(num_qubits)]
        return Quant(qubits=qubits_index, process=self)

    def execute(self):
        """Force the execution of the quantum circuit.

        This method triggers the immediate execution of the prepared quantum circuit. It is
        essential when live execution is required or when batch execution needs to be initiated.

        Example:
            >>> from ket import Process
            >>> p = Process()
            >>> # ... (quantum circuit preparation)
            >>> # Force the execution of the quantum circuit
            >>> p.execute()

        Note:
            The :meth:`~ket.base.Process.execute` method should be used when the quantum
            instructions are to be executed immediately, either in live mode or to initiate batch
            execution.
        """
        self.prepare_for_execution()

    def _get_ket_process(self):
        return self

    def get_instructions(self) -> list[dict[str, Any]]:
        """Retrieve quantum instructions from the quantum process.

        Returns:
            A list of dictionaries containing quantum instructions extracted from the process.

        The :meth:`~ket.base.Process.get_instructions` method retrieves the quantum instructions
        from the prepared quantum process. It internally calls the
        :meth:`~ket.base.Process.instructions_json` method to obtain the instructions fom the Rust
        library in JSON format, converts the byte data into a list of dictionaries, and returns the
        result.

        Note:
            Ensure that the quantum process has been appropriately prepared for execution using the
            :meth:`~ket.base.Process.prepare_for_execution` method before calling this method. The
            returned instructions provide insights into the quantum circuit and can be useful for
            debugging or analysis purposes.

        Example:
            >>> from ket import *
            >>> p = Process()
            >>> a, b = p.alloc(2)
            >>> CNOT(H(a), b)
            >>> # Get quantum instructions
            >>> instructions = p.get_instructions()
            >>> pprint(instructions)
            [{'Alloc': {'target': 0}},
             {'Alloc': {'target': 1}},
             {'Gate': {'control': [], 'gate': 'Hadamard', 'target': 0}},
             {'Gate': {'control': [0], 'gate': 'PauliX', 'target': 1}}]
        """
        write_size = self.instructions_json(
            self._instructions_buffer, self._instructions_buffer_size
        )
        if write_size.value > self._instructions_buffer_size:
            self._instructions_buffer_size = write_size.value + 1
            self._instructions_buffer = (c_uint8 * self._instructions_buffer_size)()
            return self.get_instructions()

        return loads(bytearray(self._instructions_buffer[: write_size.value]))

    def get_metadata(self) -> dict[str, Any]:
        """Retrieve metadata from the quantum process.

        Returns:
            A dictionary containing metadata information extracted from the process.

        The :meth:`~ket.base.Process.get_metadata` method retrieves metadata from the prepared
        quantum process. It internally calls the :meth:`~ket.base.Process.metadata_json` method to
        obtain the metadata in JSON format, converts the byte data into a dictionary, and returns
        the result.

        Note:
            Ensure that the quantum process has been appropriately prepared for execution using the
            :meth:`~ket.base.Process.prepare_for_execution` method before calling this method. The
            returned metadata provides information about the quantum circuit execution, including
            depth, gate count, qubit simultaneous operations, status, execution time, and timeout.

        Example:

            >>> from ket import Process
            >>> p = Process()
            >>> a, b = p.alloc(2)
            >>> CNOT(H(a), b)
            >>> # Get metadata
            >>> metadata = p.get_metadata()
            >>> pprint(metadata)
            {'depth': 2,
             'execution_time': None,
             'gate_count': {'1': 1, '2': 1},
             'qubit_simultaneous': 2,
             'status': 'Live',
             'timeout': None}
        """

        write_size = self.metadata_json(
            self._metadata_buffer, self._metadata_buffer_size
        )
        if write_size.value > self._metadata_buffer_size:
            self._metadata_buffer_size = write_size.value + 1
            self._metadata_buffer = (c_uint8 * self._metadata_buffer_size)()
            return self.get_metadata()

        return loads(bytearray(self._metadata_buffer[: write_size.value]))

    def __repr__(self) -> str:
        return f"<Ket 'Process' id={hex(id(self))}>"


class Quant:
    """List of qubits.

    This class represents a list of qubit indices within a quantum process. Direct instantiation
    of this class is not recommended. Instead, it should be created by calling the
    :meth:`~ket.base.Process.alloc` method.

    A :class:`~ket.base.Quant` serves as a fundamental quantum object where quantum operations
    should be applied.

    Example:

        .. code-block:: py

            from ket import *
            # Create a quantum process
            p = Process()

            # Allocate 2 qubits
            q1 = p.alloc(2)

            # Apply a Hadamard gates on the first qubit of `q1`
            H(q1[0])

            # Allocate more 2 qubits
            q2 = p.alloc(2)

            # Concatenate two Quant objects
            result_quant = q1 + q2
            print(result_quant)  # <Ket 'Quant' [0, 1, 2, 3] pid=0x...>

            # Use the fist qubit to control the application of
            # a Pauli X gate on the other qubits
            ctrl(result_quant[0], X)(result_quant[1:])

            # Select qubits at specific indexes
            selected_quant = result_quant.at([0, 1])
            print(selected_quant)  # <Ket 'Quant' [0, 1] pid=0x...>

            # Free all qubits in a Quant object
            result_quant.free()

            # Check if all qubits in a Quant object are free
            is_free = result_quant.is_free()
            print(is_free)  # True

    Supported operations:

    - Addition (``+``): Concatenates two :class:`~ket.base.Quant` objects.
      The processes must be the same.
    - Indexing (``[index]``): Returns a new :class:`~ket.base.Quant` object with selected qubits
      based on the provided index.
    - Iteration (``for q in qubits``): Allows iterating over qubits in a :class:`~ket.base.Quant`
      object.
    - Reversal (``reversed(qubits)``): Returns a new :class:`~ket.base.Quant` object with reversed
      qubits.
    - Context Manager (``with qubits:``): Ensures that all qubits are free at the end of the scope.
    - Length (``len(qubits)``): Returns the number of qubits in the :class:`~ket.base.Quant` object.

    """

    def __init__(self, *, qubits: list[int], process: Process):
        self.qubits = qubits
        self.process = process

    def _get_ket_process(self):
        return self.process

    def __add__(self, other: Quant) -> Quant:
        if self.process is not other.process:
            raise ValueError("Cannot concatenate qubits from different processes")
        return Quant(qubits=self.qubits + other.qubits, process=self.process)

    def at(self, index: list[int]) -> Quant:
        """Return a subset of qubits at specified indices.

        Create a new :class:`~ket.base.Quant` object with qubit references at the positions defined
        by the provided ``index`` list.

        Example:

            .. code-block:: py

                from ket import *

                # Create a quantum process
                p = Process()

                # Allocate 5 qubits
                q = p.alloc(5)

                # Select qubits at odd indices (1, 3)
                odd_qubits = q.at([1, 3])

        Args:
            index: List of indices specifying the positions of qubits to be included in the
                new :class:`~ket.base.Quant`.

        Returns:
            A new :class:`~ket.base.Quant` object containing the selected qubits.
        """

        return Quant(qubits=[self.qubits[i] for i in index], process=self.process)

    def free(self):
        r"""Release the qubits.

        This method frees the allocated qubits, assuming they are in the state
        :math:`\left|0\right>` before the call.

        Warning:
            No check is performed to verify if the qubits are in the state
            :math:`\left|0\right>`. Releasing qubits in other states cause undefined behavior.
        """
        for qubit in self.qubits:
            self.process.free_qubit(qubit)

    def is_free(self) -> bool:
        """Check if all allocated qubits are in the 'free' state.

        Returns:
            ``True`` if all qubits are free, ``False`` otherwise.
        """
        return all(
            not self.process.get_qubit_status(qubit)[0].value for qubit in self.qubits
        )

    def __reversed__(self):
        return Quant(qubits=list(reversed(self.qubits)), process=self.process)

    def __getitem__(self, key):
        qubits = self.qubits.__getitem__(key)
        return Quant(
            qubits=qubits if isinstance(qubits, list) else [qubits],
            process=self.process,
        )

    class _QuantIter:
        def __init__(self, q: Quant):
            self.q = q
            self.idx = -1
            self.size = len(q.qubits)

        def __next__(self):
            self.idx += 1
            if self.idx < self.size:
                return self.q[self.idx]
            raise StopIteration

        def __iter__(self):
            return self

    def __iter__(self):
        return self._QuantIter(self)

    def __enter__(self):
        return self

    def __exit__(
        self, type, value, tb
    ):  # pylint: disable=redefined-builtin, invalid-name
        if not self.is_free():
            raise RuntimeError("non-free quant at the end of scope")

    def __len__(self):
        return len(self.qubits)

    def __repr__(self):
        return f"<Ket 'Quant' {self.qubits} pid={hex(id(self.process))}>"


class Measurement:
    """Quantum measurement result.

    This class holds a reference for a measurement result. The result may not be available right
    after the measurement call, especially in batch execution.

    To read the value, access the attribute :attr:`~ket.base.Measurement.value`. If the value is not
    available, the measurement will return `None`; otherwise, it will return an unsigned integer.

    You can instantiate this class by calling the :func:`~ket.operations.measure` function.

    Example:

        .. code-block:: python

            from ket import *

            p = Process()
            q = p.alloc(2)
            CNOT(H(q[0]), q[1])
            result = measure(q)
            print(result.value)  # 0 or 3
    """

    def __init__(self, qubits: Quant):
        self.process = qubits.process
        self.qubits = [qubits[i : i + 64] for i in range(0, len(qubits), 64)]
        self.indexes = [
            self.process.measure(
                (c_size_t * len(qubit.qubits))(*qubit.qubits), len(qubit.qubits)
            ).value
            for qubit in self.qubits
        ]
        self._value = None

    def _get_ket_process(self):
        return self.process

    def _check(self):
        if self._value is None:
            available, values = zip(
                *(self.process.get_measurement(index) for index in self.indexes)
            )
            if all(map(lambda a: a.value, available)):
                self._value = 0
                for value, qubit in zip(values, self.qubits):
                    self._value <<= len(qubit)
                    self._value |= value.value

    @property
    def value(self) -> int | None:
        """Retrieve the measurement value if available."""
        self._check()
        return self._value

    def get(self) -> int:
        """Retrieve the measurement value.

        If the value is not available, the quantum process will execute to get the result.
        """

        self._check()
        if self._value is None:
            self.process.execute()
        return self.value

    def __repr__(self):
        return (
            f"<Ket 'Measurement' indexes={self.indexes}, "
            f"value={self.value}, pid={hex(id(self.process))}>"
        )


class QuantumState:
    """Snapshot of a quantum state.

    This class holds a reference for a snapshot of a quantum state obtained using a quantum
    simulator. The result may not be available right after the measurement call, especially in batch
    execution.

    You can instantiate this class by calling the :func:`~ket.operations.dump` function.

    :Example:

        .. code-block:: python

            from ket import *

            p = Process()
            a, b = p.alloc(2)
            with around(cat(kron(H, I), CNOT), a, b):
                Y(a)
                inside = dump(a+b)

            outside = dump(a+b)
            print(inside.show())
            # |01⟩    (50.00%)
            #  -0.707107i     ≅     -i/√2
            # |10⟩    (50.00%)
            #  0.707107i     ≅      i/√2

            print(outside.show())
            # |11⟩    (100.00%)
            #  -1.000000i     ≅     -i/√1

    Args:
        qubits: Qubits from which to capture a quantum state snapshot.
    """

    def __init__(self, qubits: Quant):
        self.qubits = qubits
        self.process = qubits.process
        self.index = self.process.dump(
            (c_size_t * len(qubits.qubits))(*qubits.qubits), len(qubits.qubits)
        ).value
        self.size = len(qubits)
        self._states = None

    def _get_ket_process(self):
        return self.process

    def _check(self):
        if self._states is None:
            available, size = self.process.get_dump_size(self.index)
            if available.value:
                states = defaultdict(complex)
                for i in range(size.value):
                    state, state_size, amp_real, amp_imag = self.process.get_dump(
                        self.index, i
                    )
                    state = int(
                        "".join(f"{state[j]:064b}" for j in range(state_size.value)), 2
                    )
                    amplitude = complex(amp_real.value, amp_imag.value)
                    states[state] += amplitude

                p = sum(map(lambda a: abs(a) ** 2, states.values()))
                if abs(p - 1.0) < 1e-10:
                    self._states = dict(states)
                else:
                    self._states = {
                        state: amplitude / sqrt(p)
                        for state, amplitude in states.items()
                    }

    @property
    def states(self) -> dict[int, complex] | None:
        """Get the quantum state.

        Returns a dictionary mapping base states to their corresponding probability amplitudes.

        Returns:
            The quantum state, or None if the quantum state information is not available.
        """

        self._check()
        return self._states

    def get(self) -> dict[int, complex]:
        """Get the quantum state.

        If the quantum state is not available, the quantum process will execute to get the result.
        """

        self._check()
        if self._states is None:
            self.process.execute()
        return self.states

    @property
    def probabilities(self) -> dict[int, float] | None:
        """Get the measurement probabilities.

        Returns a dictionary mapping base states to their corresponding measurement probabilities.

        Returns:
            The measurement probabilities, or None if the quantum state information is not
            available.
        """
        self._check()
        if self._states is None:
            return None
        return dict(map(lambda a: (a[0], abs(a[1]) ** 2), self._states.items()))

    def sample(self, shots=4096, seed=None) -> dict[int, int] | None:
        """Get the quantum execution shots.

        The parameters ``shots`` and ``seed`` are used to generate the sample from the quantum state
        snapshot.

        Args:
            shots: Number of shots. Defaults to 4096.
            seed: Seed for the RNG.

        Returns:
            A dictionary mapping measurement outcomes to their frequencies in the generated sample,
            or None if the quantum state information is not available.
        """

        self._check()
        if self._states is None:
            return None

        rng = Random(seed)
        shots = rng.choices(list(self.states), list(self.probabilities), k=shots)
        result = {}
        for state in shots:
            if state not in result:
                result[state] = 1
            else:
                result[state] += 1
        return result

    def show(self, format_str: str | None = None) -> str | None:
        r"""Return the quantum state as a string.

        Use the format string to change the print format of the basis states:

        * ``i``: print the state in the decimal base
        * ``b``: print the state in the binary base (default)
        * ``i|b<n>``: separate the ``n`` first qubits; the remaining print in the binary base
        * ``i|b<n1>:i|b<n2>[:i|b<n3>...]``: separate the ``n1, n2, n3, ...`` first qubits

        :Example:

            .. code-block:: py

                from ket import *

                p = Process()

                q = p.alloc(19)
                X(ctrl(H(q[0]), X, q[1:])[1::2])
                d = dump(q)

                print(d.show('i'))
                # |87381⟩ (50.00%)
                #  0.707107               ≅      1/√2
                # |436906⟩        (50.00%)
                #  0.707107               ≅      1/√2
                print(d.show('b'))
                # |0010101010101010101⟩   (50.00%)
                #  0.707107               ≅      1/√2
                # |1101010101010101010⟩   (50.00%)
                #  0.707107               ≅      1/√2
                print(d.show('i4'))
                # |2⟩|101010101010101⟩    (50.00%)
                #  0.707107               ≅      1/√2
                # |13⟩|010101010101010⟩   (50.00%)
                #  0.707107               ≅      1/√2
                print(d.show('b5:i4'))
                # |00101⟩|5⟩|0101010101⟩  (50.00%)
                #  0.707107               ≅      1/√2
                # |11010⟩|10⟩|1010101010⟩ (50.00%)
                #  0.707107               ≅      1/√2

        Args:
            format_str: Format string that matches ``(i|b)\d*(:(i|b)\d+)*``.

        Returns:
            The formatted quantum state as a string, or None if the quantum state information is not
            available.
        """

        self._check()
        if self._states is None:
            return None

        if format_str is not None:
            if format_str in ("b", "i"):
                format_str += str(self.size)
            fmt = []
            count = 0
            for b, size in map(lambda f: (f[0], int(f[1:])), format_str.split(":")):
                fmt.append((b, count, count + size))
                count += size
            if count < self.size:
                fmt.append(("b", count, self.size))
        else:
            fmt = [("b", 0, self.size)]

        def fmt_ket(state, begin, end, f):
            return (
                f"|{state[begin:end]}⟩"
                if f == "b"
                else f"|{int(state[begin:end], base=2)}⟩"
            )

        def state_amp_str(state, amp):
            dump_str = "".join(
                fmt_ket(f"{state:0{self.size}b}", b, e, f) for f, b, e in fmt
            )
            dump_str += f"\t({100*abs(amp)**2:.2f}%)\n"
            real = abs(amp.real) > 1e-10
            real_l0 = amp.real < 0

            imag = abs(amp.imag) > 1e-10
            imag_l0 = amp.imag < 0

            sqrt_dem = 1 / abs(amp) ** 2
            use_sqrt = abs(round(sqrt_dem) - sqrt_dem) < 0.001
            use_sqrt = use_sqrt and (
                (abs(abs(amp.real) - abs(amp.imag)) < 1e-6) or (real != imag)
            )
            sqrt_dem = f"/√{round(1/abs(amp)**2)}"

            if real and imag:
                sqrt_dem = f"/√{round(2*(1/abs(amp)**2))}"
                sqrt_num = ("(-1" if real_l0 else " (1") + ("-i" if imag_l0 else "+i")
                sqrt_str = (
                    f"\t≅ {sqrt_num}){sqrt_dem}"
                    if use_sqrt and (abs(amp.real) - abs(amp.real) < 1e-10)
                    else ""
                )
                dump_str += f"{amp.real:9.6f}{amp.imag:+.6f}i" + sqrt_str
            elif real:
                sqrt_num = "  -1" if real_l0 else "   1"
                sqrt_str = f"\t≅   {sqrt_num}{sqrt_dem}" if use_sqrt else ""
                dump_str += f"{amp.real:9.6f}       " + sqrt_str
            else:
                sqrt_num = "  -i" if imag_l0 else "   i"
                sqrt_str = f"\t≅   {sqrt_num}{sqrt_dem}" if use_sqrt else ""
                dump_str += f" {amp.imag:17.6f}i" + sqrt_str

            return dump_str

        return "\n".join(
            state_amp_str(state, amp)
            for state, amp in sorted(self._states.items(), key=lambda k: k[0])
        )

    def __repr__(self):
        return f"<Ket 'QuantumState' index={self.index}, pid={hex(id(self.process))}>"


class Samples:
    """Quantum state measurement samples.

    This class holds a reference for a measurement sample result. The result may not be available
    right after the sample call, especially in batch execution.

    To read the value, access the attribute :attr:`~ket.base.Sample.value`. If the value is not
    available, the measurement will return `None`; otherwise, it will return a dictionary mapping
    measurement outcomes to their respective counts.

    You can instantiate this class by calling the :func:`~ket.operations.sample` function.

    Example:

        .. code-block:: py

            from ket import *

            p = Process()
            q = p.alloc(2)
            CNOT(H(q[0]), q[1])
            results = sample(q)

            print(results.value)
            # {0: 1042, 3: 1006}

    Args:
        qubits: Qubits for which the measurement samples are obtained.
        shots: Number of measurement shots (default is 2048).

    """

    def __init__(self, qubits: Quant, shots: int = 2048):
        self.qubits = qubits
        self.process = qubits.process
        self.index = self.process.sample(
            (c_size_t * len(qubits.qubits))(*qubits.qubits), len(qubits.qubits), shots
        ).value
        self._value = None

    def _check(self):
        if self._value is None:
            (
                available,
                states,
                count,
                size,
            ) = self.process.get_sample(self.index)
            if available.value:
                self._value = dict(zip(states[: size.value], count[: size.value]))

    @property
    def value(self) -> dict[int, int] | None:
        """Retrieve the measurement samples if available."""
        self._check()
        return self._value

    def get(self) -> dict[int, int]:
        """Retrieve the measurement samples.

        If the value is not available, the quantum process will execute to get the result.
        """

        self._check()
        if self._value is None:
            self.process.execute()
        return self.value

    def __repr__(self) -> str:
        return f"<Ket 'Samples' index={self.index}, pid={hex(id(self.process))}>"
