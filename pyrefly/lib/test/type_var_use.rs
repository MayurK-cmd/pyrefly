/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::test::util::TestEnv;
use crate::test::util::shape_extensions_env;
use crate::testcase;

// https://github.com/pyrefly-org/pyrefly/issues/3656
testcase!(
    test_type_var_used_once,
    TestEnv::new().enable_unnecessary_type_var_error(),
    r#"
from typing import TypeVar

T = TypeVar("T")

def pep695[S](x: S) -> None: ...  # E: Type variable `S` appears only once in the signature of `pep695`
def legacy(x: T) -> None: ...  # E: Type variable `T` appears only once in the signature of `legacy`
def bounded[S: int](x: S) -> None: ...  # E: Type variable `S` appears only once in the signature of `bounded`
def union_member[S](x: S | None) -> None: ...  # E: Type variable `S` appears only once in the signature of `union_member`
def return_only[S]() -> S: ...  # E: Type variable `S` appears only once in the signature of `return_only`

class A:
    def method(self, x: T) -> None: ...  # E: Type variable `T` appears only once in the signature of `method`
"#,
);

testcase!(
    test_type_var_used_once_exemptions,
    TestEnv::new().enable_unnecessary_type_var_error(),
    r#"
from typing import Callable, Generic, TypeVar

K = TypeVar("K")

def repeated[S](x: S) -> S: ...
def two_params[S](x: S, y: S) -> None: ...
def nested_param[S](x: list[S]) -> None: ...
def nested_return[S]() -> list[S]: ...
def constrained[S: (int, str)](x: S) -> None: ...
def with_default[S = int](x: S) -> None: ...
def param_spec[**P](f: Callable[P, None]) -> None: ...
def type_var_tuple[*Ts](*args: *Ts) -> None: ...
def inferred_return[S](x: S):
    return x

class Box(Generic[K]):
    def class_scoped(self, x: K) -> None: ...
    def implicit_self(self) -> None: ...
"#,
);

testcase!(
    test_int_var_used_once,
    shape_extensions_env().enable_unnecessary_type_var_error(),
    r#"
from shape_extensions import IntTuple, IntVar

class Tensor[Shape: IntTuple]:
    shape: Shape

def rank_one[BatchSize: IntVar](x: Tensor[[BatchSize]]) -> None: ...
def make_rank_one[BatchSize: IntVar]() -> Tensor[[BatchSize]]: ...
"#,
);

// https://github.com/pyrefly-org/pyrefly/issues/3656
// A function whose body is `...` treats a `...` default as an unknown value, so a type
// variable that only such parameters mention is solved to `Unknown` when they are omitted.
testcase!(
    test_possibly_unsolved_type_var,
    r#"
from typing import Callable, overload, reveal_type

def omitted[T](value: T = ...) -> T: ...  # E: Type variable `T` may go unsolved if a call to `omitted` omits argument `value`
x = omitted()
reveal_type(x)  # E: revealed type: Unknown

def keyword_only[T](*, value: T = ...) -> list[T]: ...  # E: Type variable `T` may go unsolved if a call to `keyword_only` omits argument `value`
def two_optional[T](a: T = ..., b: list[T] = ...) -> T: ...  # E: Type variable `T` may go unsolved if a call to `two_optional` omits argument `a`
def param_spec[**P](f: Callable[P, None] = ...) -> Callable[P, None]: ...  # E: Type variable `P` may go unsolved if a call to `param_spec` omits argument `f`

@overload
def foo(value: int) -> None: ...
@overload
def foo[T](value: T = ...) -> T: ...  # E: Type variable `T` may go unsolved if a call to `foo` omits argument `value`
def foo(value: object = None) -> object: ...
y = foo()
reveal_type(y)  # E: revealed type: Unknown
"#,
);

testcase!(
    test_possibly_unsolved_type_var_exemptions,
    r#"
from typing import reveal_type

def real_default[T](value: T = 1) -> T: ...
reveal_type(real_default())  # E: revealed type: int

# With a real body, `...` is the runtime default value.
def with_body[T](value: T = ...) -> T:
    return value
reveal_type(with_body())  # E: revealed type: EllipsisType

def type_param_default[T = int](value: T = ...) -> T: ...
reveal_type(type_param_default())  # E: revealed type: int

def required_elsewhere[T](a: T, b: T = ...) -> T: ...
def varargs[T](*args: T) -> T: ...
def not_in_return[T](a: T = ..., b: T = ...) -> None: ...
"#,
);

testcase!(
    test_possibly_unsolved_int_var,
    shape_extensions_env(),
    r#"
from shape_extensions import IntTuple, IntVar

class Tensor[Shape: IntTuple]:
    shape: Shape

def maybe_batched[BatchSize: IntVar](x: Tensor[[BatchSize]] = ...) -> Tensor[[BatchSize]]: ...  # E: Type variable `BatchSize` may go unsolved if a call to `maybe_batched` omits argument `x`
"#,
);
