from __future__ import annotations
#  Copyright 2020, 2021 Evandro Chagas Ribeiro da Rosa <evandro.crr@posgrad.ufsc.br>
#  Copyright 2020, 2021 Rafael de Santiago <r.santiago@ufsc.br>
# 
#  Licensed under the Apache License, Version 2.0 (the "License");
#  you may not use this file except in compliance with the License.
#  You may obtain a copy of the License at
# 
#      http://www.apache.org/licenses/LICENSE-2.0
# 
#  Unless required by applicable law or agreed to in writing, software
#  distributed under the License is distributed on an "AS IS" BASIS,
#  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#  See the License for the specific language governing permissions and
#  limitations under the License.

from ..ket import ctrl_begin, ctrl_end, X
from ..types import quant
from typing import Iterable, Callable, Optional, Any
from functools import reduce
from operator import add

def _create_mask(on_state : Optional[int | Iterable[int]], length : int) -> list[int]:
    """Create a mask for ctrl and control"""

    if on_state is None:
        return None
    elif hasattr(on_state, '__iter__'):
        if len(on_state) != length:
            raise ValueError(f"'on_state' received a list of length {len(on_state)} to use on {length} qubits")
        return on_state
    else:
        if length < on_state.bit_length():
            raise ValueError(f"To control 'on_state' {on_state} you need at least {on_state.bit_length()} qubits")
        return [int(i) for i in f"{{:0{length}b}}".format(on_state)]

def _apply_mask(mask : Iterable[int], q : quant):
    """Flip qubit q[i] if mask[i] == 0"""
    
    if mask is not None:
        for i, q in zip(mask, q):
            if i == 0:
                X(q)

class control:
    r"""Open a controlled-scope
    
    Inside a ``with control`` scope, the qubits of ``ctr`` control every
    quantum operation, only execution if all qubits are in the estate
    :math:`\left|1\right>`.

    Optionally, you can change the state when applying the quantum operation
    with the Keyword Argument ``on_state``.  If ``on_state`` is an ``int``, the
    number of qubits needs to be equal to or greater than the bit length of
    ``on_state``.  Else if ``on_sate`` is a ``List[int]``, its length needs to
    be equal to the number of qubits.

    For example, execute when the qubits are in the state
    :math:`\left|\left[\dots0\right]11\right>`, use ``with control(ctr, on_state=3):``
    or ``with control(ctr, on_state=[0, 1, 1]):`` if ``ctr`` has exactly 3 qubits.
    
    :Usage:

    .. code-block:: ket
    
        with control(*ctr[, on_state]):
            ...

    :Example:

    .. code-block:: ket

        c = quant(2)
        a, b = quant(2)

        # CNOT c[0] a
        with control(c[0]):
            X(a)
        
        # Toffoli c[0] c[1] a
        with control(c):
            X(a)

        # CSWAP c[0] a b
        with control(c[0]):
            swap(a, b)

    Args:
        ctr: Control qubits.
        on_state: Change the control state. 
    """

    def __init__(self, *ctr : quant, on_state : Optional[int | Iterable[int]] = None):
        self.ctr = reduce(add, ctr)
        self.mask = _create_mask(on_state, len(self.ctr))

    def __enter__ (self):
        _apply_mask(self.mask, self.ctr)
        ctrl_begin(self.ctr)
     
    def __exit__ (self, type, value, tb):
        ctrl_end()
        _apply_mask(self.mask, self.ctr)
            
def _ctrl(control  : quant | Iterable[quant], 
          func     : Callable | Iterable[Callable],
          *args, 
          on_state : Optional[int | list[int]] = None,
          **kwargs) -> Any:
    """Call Callable with controll-qubits"""

    ret = []
    mask = _create_mask(on_state, len(control))

    _apply_mask(mask, control)
    ctrl_begin(control)
    
    if hasattr(func, '__iter__'): 
        for f in func:
            ret.append(f(*args, **kwargs))
    else:
        ret = func(*args, **kwargs)

    ctrl_end()
    _apply_mask(mask, control)
    
    return ret

def _qubit_for_ctrl(qubits : quant | Iterable[quant] | slice | int | Iterable[int]) -> tuple[Callable[[quant], quant] | quant, bool]:
    """Get qubits for ctrl"""

    if any(isinstance(qubits, tp) for tp in [slice, int]):
        return lambda q : q[qubits], True
    elif hasattr(qubits, '__iter__') and all(isinstance(i, int) for i in qubits):
        return lambda q : q.at(qubits), True
    elif hasattr(qubits, '__iter__') and all(isinstance(i, quant) for i in qubits):
        return reduce(add, qubits), False
    else:
        return qubits, False

def ctrl(control    : quant | Iterable[quant] | slice | int | Iterable[int], 
         func       : Callable | Iterable[Callable] , 
         *args,    
         on_state   : Optional[int | Iterable[int]] = None,
         target     : Optional[slice | int | Iterable[int]] = None,
         later_call : bool = False,
         **kwargs) -> Callable | Any :
    r"""Add controll-qubits to a Callable

    :Call with control qubits:

    * ``control`` type must be :class:`~ket.types.quant` or ``Iterable[quant]``
    * ``func`` type must be ``Callable`` or ``Iterable[Callable]``

    .. code-block:: ket

        ret1 = ctrl(control_qubits, func, *args, **kwargs)
        # Equivalent to:
        # with control(control_qubits):
        #     ret1 = func(*args, **kwargs)

        ret2 = ctrl([q0, q1, q2, ...], func, *args, **kwargs)
        # Equivalent to:
        # with control(*control):
        #     ret2 = func(*args, **kwargs)
        
        ret3 = ctrl(control_qubits, [f0, f1, f2, ...], *args, **kwargs)
        # Equivalent to:
        # ret3 = []
        # with control(control_qubits):
        #     for f in func:
        #         ret3.append(f(*args, **kwargs))
 
    :Create controlled-operation:

    1. If the keyword argument ``later_call`` is ``True``, return a
    ``Callable[[], Any]``:

    .. code-block:: ket

        ctrl_func = ctrl(control_qubits, func, *args, **kwargs, later_call=True)
        # Equivalent to:
        # ctrl_func = lambda : ctrl(control_qubits, func, *args, **kwargs)

    Example:

    .. code-block:: ket
        :emphasize-lines: 8

        def increment(q):
            if len(q) > 1:
                ctrl(q[-1], increment, q[:-1])
            X(q[-1])

        size = ceil(log2(len(inputs)))+1
        with quant(size) as inc:
            with around(ctrl(q , increment, inc, later_call=True) for q in inputs):
                with control(inc, on_state=len(inputs)//2):
                    X(output)
            inc.free()

    2. If ``control`` and ``target`` type is ``int``, ``slice``, or
    ``Iterable[int]``,  return a ``Callable[[quant], Any]``:

    .. code-block:: ket

        ctrl_func = ctrl(ctrl_index, func, target_index)
        # Equivalent to:
        # ctrl_func = lambda q : ctrl(q[ctrl_index], func, q[target_index])

    Example:

    .. code-block:: ket

        with around(ctrl(0, X, slice(1, None)), q): # ctrl(q[0], X, q[1:]) 
            H(q[0])                                 # H(q[0])
                                                    # ctrl(q[0], X, q[1:])
 
    Args:
        control: Control qubits.
        func: Functions to add control.
        on_state: Change the control state, same as for :class:`~ket.standard.control`. 
        target: Target qubits to create controlled-operations.
        later_call: If ``True``, do not execute and return a ``Callable[[], Any]``.
    """

    control, create_gate = _qubit_for_ctrl(control)
    if create_gate:
        target, _ = _qubit_for_ctrl(*args if target is None else target)
        def _ctrl_gate(q : quant) -> quant:
            _ctrl(control(q), func, target(q), on_state=on_state)
            return q
        return _ctrl_gate
    elif later_call:
        return lambda : _ctrl(control, func, *args, on_state=on_state, **kwargs)
    else:
        return _ctrl(control, func, *args, on_state=on_state, **kwargs)
