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

from ..ket import label, branch, jump
from ..types import future

__all__ = ['_ket_is_future', '_ket_if', '_ket_if_else', '_ket_next']

def _ket_is_future(obj) -> bool:
    return isinstance(obj, future)

def _ket_if(test : future) -> label:
    if_then = label('if.then')
    if_end  = label('if.end') 
    branch(test, if_then, if_end)
    if_then.begin()
    return if_end

def _ket_if_else(test : future) -> tuple[label]:
    if_then = label('if.then')
    if_else = label('if.else') 
    if_end  = label('if.end') 
    branch(test, if_then, if_else)
    if_then.begin()
    return if_else, if_end

def _ket_next(end : label):
    jump(end)
    end.begin()