use std::{convert::Infallible, ops::ControlFlow};

use try_macro::{try_macro, FromResidual, Try};

struct Inner;
#[derive(Debug, PartialEq, Eq)]
struct Outer;
impl From<Inner> for Outer {
    fn from(_: Inner) -> Self {
        Outer
    }
}

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

fn some() -> Option<u8> { Some(3) }
fn none() -> Option<u8> { None }
fn err() -> Result<u8, Inner> { Err(Inner) }
fn ok() -> Result<u8, Inner> { Ok(3) }
fn left() -> Either<&'static str, u8> { Either::Left("err") }
fn right() -> Either<&'static str, u8> { Either::Right(3) }

#[try_macro]
fn apply_option() -> Option<i32> {
    let x = none()?;
    Some(x as i32)
}

#[try_macro]
fn apply_option_some() -> Option<i32> {
    let x = some()?;
    Some(x as i32)
}

#[try_macro]
fn apply_result() -> Result<i32, Outer> {
    let x = err()?;
    Ok(x as i32)
}

#[try_macro]
fn apply_result_ok() -> Result<i32, Outer> {
    let x = ok()?;
    Ok(x as i32)
}

#[try_macro]
fn apply_either() -> Either<&'static str, i32> {
    let x = left()?;
    Either::Right(x as i32)
}

#[try_macro]
fn apply_either_right() -> Either<&'static str, i32> {
    let x = right()?;
    Either::Right(x as i32)
}

#[test]
fn it_works() {
    assert_eq!(apply_option(), None);
    assert_eq!(apply_option_some(), Some(3));
    assert_eq!(apply_result(), Err(Outer));
    assert_eq!(apply_result_ok(), Ok(3));
    assert_eq!(apply_either(), Either::Left("err"));
    assert_eq!(apply_either_right(), Either::Right(3));
}
