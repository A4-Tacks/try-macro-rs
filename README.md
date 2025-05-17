Replace `?` with custom `Try::branch` in the macro

# Examples
```rust
use std::convert::Infallible;
use std::ops::ControlFlow;
use try_macro::{Try, FromResidual, try_macro};

fn left() -> Either<&'static str, u8> { Either::Left("err") }
fn right() -> Either<&'static str, u8> { Either::Right(3) }

#[try_macro]
fn either_left() -> Either<&'static str, i32> {
    let x = left()?;
    Either::Right(x as i32)
}

#[try_macro]
fn either_right() -> Either<&'static str, i32> {
    let x = right()?;
    Either::Right(x as i32)
}

assert_eq!(either_left(), Either::Left("err"));
assert_eq!(either_right(), Either::Right(3));

#[derive(Debug, PartialEq, Eq)]
enum Either<L, R> {
    Left(L),
    Right(R),
}
impl<L, R> Try for Either<L, R> {
    type Output = R;
    type Residual = Either<L, Infallible>;

    fn from_output(output: Self::Output) -> Self {
        Self::Right(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Either::Left(left) => ControlFlow::Break(Either::Left(left)),
            Either::Right(right) => ControlFlow::Continue(right),
        }
    }
}
impl<L, R> FromResidual for Either<L, R> {
    fn from_residual(residual: <Self as Try>::Residual) -> Self {
        match residual {
            Either::Left(left) => Self::Left(left),
        }
    }
}
```

Expand to like:

```rust,ignore
fn either_left() -> Either<&'static str, i32> {
    let x = match ::try_macro::Try::branch(left()) {
        ::core::ops::ControlFlow::Continue(value) => value,
        ::core::ops::ControlFlow::Break(err) => {
            return ::try_macro::FromResidual::from_residual(err);
        }
    };
    Either::Right(x as i32)
}
```

Provided easy-to-use macros for [`stable_try_trait_v2`](https://crates.io/crates/stable_try_trait_v2)
