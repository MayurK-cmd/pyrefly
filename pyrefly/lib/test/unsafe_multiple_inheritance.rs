/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::test::util::TestEnv;
use crate::testcase;

// These tests follow basedpyright's `reportUnsafeMultipleInheritance` sample
// (packages/pyright-internal/src/tests/samples/multipleInheritance.py) and pin its
// semantics: https://github.com/pyrefly-org/pyrefly/issues/3726.

fn env() -> TestEnv {
    TestEnv::new().enable_unsafe_multiple_inheritance_error()
}

testcase!(
    test_unsafe_multiple_inheritance_init,
    env(),
    r#"
class A:
    def __init__(self) -> None:
        pass

class B:
    def __init__(self) -> None:
        pass

class C: ...

class E(B): ...

class F(A, B): ...  # E: Multiple inheritance is not allowed because the following base classes have `__init__` methods that may not get called: `B`
class G(A, E): ...  # E: may not get called: `B`
class H(A, C): ...
class I(B, C): ...
"#,
);

testcase!(
    test_unsafe_multiple_inheritance_reports_every_extra_base,
    env(),
    r#"
class A:
    def __init__(self) -> None: ...
class B:
    def __init__(self) -> None: ...
class D:
    def __init__(self) -> None: ...

class X(A, B, D): ...  # E: may not get called: `B`, `D`
"#,
);

testcase!(
    test_unsafe_multiple_inheritance_cooperative_init_is_not_exempt,
    env(),
    r#"
class A:
    def __init__(self) -> None:
        super().__init__()

class B:
    def __init__(self) -> None:
        super().__init__()

class F(A, B): ...  # E: may not get called: `B`
"#,
);

testcase!(
    test_unsafe_multiple_inheritance_single_constructor,
    env(),
    r#"
class R: ...

class S:
    def __init__(self, a: int):
        super().__init__()

class T(R, S):
    pass
"#,
);

// basedpyright's documentation mentions `__new__`, but its check only reports `__init__`.
testcase!(
    test_unsafe_multiple_inheritance_ignores_new,
    env(),
    r#"
from enum import Enum
from typing import Self

class A:
    def __init__(self) -> None: ...

class N:
    def __new__(cls) -> Self:
        return super().__new__(cls)

class AN(A, N): ...
class NA(N, A): ...
class IntA(int, A): ...
class StrEnum(str, Enum): ...
"#,
);

testcase!(
    test_unsafe_multiple_inheritance_diamond,
    env(),
    r#"
class Up:
    def __init__(self): ...

class Left(Up): ...
class Right(Up): ...
class Down(Left, Right): ...

class MyError(ValueError, KeyError): ...

class NewAfterSharedInit(Up):
    def __new__(cls) -> "NewAfterSharedInit":
        return super().__new__(cls)
class DownWithNew(Left, NewAfterSharedInit): ...
"#,
);

// basedpyright compares specialized classes by object identity, so it reports a diamond through a
// generic base when the two paths specialize it in different class definitions (`LeftRight`
// below). We treat that diamond like any other.
testcase!(
    test_unsafe_multiple_inheritance_generic_diamond,
    env(),
    r#"
from typing import Generic, TypeVar

T = TypeVar("T")

class Up(Generic[T]):
    def __init__(self) -> None: ...

class Mid(Up[int]): ...
class MidLeft(Mid): ...
class MidRight(Mid): ...
class MidDown(MidLeft, MidRight): ...

class Left(Up[int]): ...
class Right(Up[int]): ...
class LeftRight(Left, Right): ...
"#,
);

testcase!(
    test_unsafe_multiple_inheritance_dataclasses,
    env(),
    r#"
from dataclasses import dataclass

class A:
    def __init__(self) -> None: ...

class C: ...

@dataclass
class J: ...

@dataclass
class K: ...

@dataclass
class L:
    def __init__(self) -> None: ...

class M(J, K): ...
class N(J, L): ...  # E: may not get called: `L`

@dataclass
class U(A, C): ...  # E: may not get called: `A`

@dataclass(init=False)
class V(A, C): ...

# Like basedpyright, a base counts as having a generated `__init__` only when it is itself a
# dataclass, so `A` is not reported even though `J.__init__` runs instead of `A.__init__`.
class InheritsJ(J): ...
class InheritsJA(InheritsJ, A): ...

# Like basedpyright, a dataclass counts as having a generated `__init__` unless it passes
# `init=False`, even when it defines `__init__` itself.
@dataclass
class X(A, C):  # E: may not get called: `A`
    def __init__(self) -> None: ...
"#,
);

// basedpyright reports this pattern for pydantic models combined with `ABC` or a `Protocol`:
// https://github.com/DetachHead/basedpyright/issues/1817.
testcase!(
    test_unsafe_multiple_inheritance_dataclass_transform,
    env(),
    r#"
from abc import ABC
from typing import Any, Protocol, dataclass_transform

@dataclass_transform()
class ModelBase:
    def __init__(self, **data: Any) -> None: ...

class P(Protocol): ...

class X(ModelBase, ABC): ...  # E: may not get called: `ModelBase`
class Y(ModelBase, P): ...  # E: may not get called: `ModelBase`
"#,
);

testcase!(
    test_unsafe_multiple_inheritance_ignored_bases,
    env(),
    r#"
from typing import Any, Generic, Protocol, TypedDict, TypeVar

T = TypeVar("T")

class A:
    def __init__(self) -> None: ...

class P(Protocol):
    def __init__(self) -> None: ...

class O(TypedDict): ...
class Q(TypedDict): ...

class OQ(O, Q): ...
class AG(A, Generic[T]): ...
class AO(A, object): ...

# Only `Protocol` itself is ignored, not protocol classes.
class AP(A, P): ...  # E: may not get called: `P`

def f(base: Any) -> None:
    class AAny(A, base): ...
"#,
);

fn env_with_stub() -> TestEnv {
    let mut env = env();
    env.add_with_path(
        "stub",
        "stub.pyi",
        r#"
class A:
    def __init__(self) -> None: ...
class B:
    def __init__(self) -> None: ...
class F(A, B): ...  # E: may not get called: `B`
"#,
    );
    env
}

testcase!(
    test_unsafe_multiple_inheritance_stub,
    env_with_stub(),
    r#"
import stub
"#,
);
