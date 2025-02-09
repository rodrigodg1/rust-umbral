//! This module is an adapter to the ECC backend.
//! `elliptic_curves` has a somewhat unstable API,
//! and we isolate all the related logic here.

use core::default::Default;
use core::ops::{Add, Mul, Sub};
use digest::Digest;
use ecdsa::hazmat::FromDigest;
use elliptic_curve::ff::PrimeField;
use elliptic_curve::sec1::{CompressedPointSize, EncodedPoint, FromEncodedPoint, ToEncodedPoint};
use elliptic_curve::NonZeroScalar;
use elliptic_curve::{AffinePoint, Curve, ProjectiveArithmetic, Scalar};
use generic_array::GenericArray;
use k256::Secp256k1;
use rand_core::OsRng;
use subtle::CtOption;

use crate::traits::{
    DeserializableFromArray, DeserializationError, RepresentableAsArray, SerializableToArray,
};

pub(crate) type CurveType = Secp256k1;

type BackendScalar = Scalar<CurveType>;
pub(crate) type BackendNonZeroScalar = NonZeroScalar<CurveType>;

// We have to define newtypes for scalar and point here because the compiler
// is not currently smart enough to resolve `BackendScalar` and `BackendPoint`
// as specific types, so we cannot implement local traits for them.
//
// They also have to be public because Rust isn't smart enough to understand that
//     type PointSize = <Point as RepresentableAsArray>::Size;
// isn't leaking the `Point` (probably because type aliases are just inlined).

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CurveScalar(BackendScalar);

impl CurveScalar {
    pub(crate) fn from_backend_scalar(scalar: &BackendScalar) -> Self {
        Self(*scalar)
    }

    pub(crate) fn to_backend_scalar(&self) -> BackendScalar {
        self.0
    }

    pub(crate) fn invert(&self) -> CtOption<Self> {
        self.0.invert().map(Self)
    }

    pub(crate) fn one() -> Self {
        Self(BackendScalar::one())
    }

    pub(crate) fn is_zero(&self) -> bool {
        self.0.is_zero().into()
    }

    /// Generates a random non-zero scalar (in nearly constant-time).
    pub(crate) fn random_nonzero() -> CurveScalar {
        Self(*BackendNonZeroScalar::random(&mut OsRng))
    }

    pub(crate) fn from_digest(
        d: impl Digest<OutputSize = <CurveScalar as RepresentableAsArray>::Size>,
    ) -> Self {
        Self(BackendScalar::from_digest(d))
    }
}

impl Default for CurveScalar {
    fn default() -> Self {
        Self(BackendScalar::default())
    }
}

impl RepresentableAsArray for CurveScalar {
    // Currently it's the only size available.
    // A separate scalar size may appear in later versions of `elliptic_curve`.
    type Size = <CurveType as Curve>::FieldSize;
}

impl SerializableToArray for CurveScalar {
    fn to_array(&self) -> GenericArray<u8, Self::Size> {
        self.0.to_bytes()
    }
}

impl DeserializableFromArray for CurveScalar {
    fn from_array(arr: &GenericArray<u8, Self::Size>) -> Result<Self, DeserializationError> {
        Scalar::<CurveType>::from_repr(*arr)
            .map(Self)
            .ok_or(DeserializationError::ConstructionFailure)
    }
}

type BackendPoint = <CurveType as ProjectiveArithmetic>::ProjectivePoint;
type BackendPointAffine = AffinePoint<CurveType>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CurvePoint(BackendPoint);

impl CurvePoint {
    pub(crate) fn from_backend_point(point: &BackendPoint) -> Self {
        Self(*point)
    }

    pub(crate) fn generator() -> Self {
        Self(BackendPoint::generator())
    }

    pub(crate) fn identity() -> Self {
        Self(BackendPoint::identity())
    }

    pub(crate) fn to_affine_point(&self) -> BackendPointAffine {
        self.0.to_affine()
    }

    pub(crate) fn from_compressed_array(
        arr: &GenericArray<u8, CompressedPointSize<CurveType>>,
    ) -> Option<Self> {
        let ep = EncodedPoint::<CurveType>::from_bytes(arr.as_slice()).ok()?;
        let cp_opt: Option<BackendPoint> = BackendPoint::from_encoded_point(&ep);
        cp_opt.map(Self)
    }

    fn to_compressed_array(&self) -> GenericArray<u8, CompressedPointSize<CurveType>> {
        *GenericArray::<u8, CompressedPointSize<CurveType>>::from_slice(
            self.0.to_affine().to_encoded_point(true).as_bytes(),
        )
    }
}

impl Add<&CurveScalar> for &CurveScalar {
    type Output = CurveScalar;

    fn add(self, other: &CurveScalar) -> CurveScalar {
        CurveScalar(self.0.add(&(other.0)))
    }
}

impl Add<&CurvePoint> for &CurvePoint {
    type Output = CurvePoint;

    fn add(self, other: &CurvePoint) -> CurvePoint {
        CurvePoint(self.0.add(&(other.0)))
    }
}

impl Sub<&CurveScalar> for &CurveScalar {
    type Output = CurveScalar;

    fn sub(self, other: &CurveScalar) -> CurveScalar {
        CurveScalar(self.0.sub(&(other.0)))
    }
}

impl Mul<&CurveScalar> for &CurvePoint {
    type Output = CurvePoint;

    fn mul(self, other: &CurveScalar) -> CurvePoint {
        CurvePoint(self.0.mul(&(other.0)))
    }
}

impl Mul<&CurveScalar> for &CurveScalar {
    type Output = CurveScalar;

    fn mul(self, other: &CurveScalar) -> CurveScalar {
        CurveScalar(self.0.mul(&(other.0)))
    }
}

impl RepresentableAsArray for CurvePoint {
    type Size = CompressedPointSize<CurveType>;
}

impl SerializableToArray for CurvePoint {
    fn to_array(&self) -> GenericArray<u8, Self::Size> {
        self.to_compressed_array()
    }
}

impl DeserializableFromArray for CurvePoint {
    fn from_array(arr: &GenericArray<u8, Self::Size>) -> Result<Self, DeserializationError> {
        Self::from_compressed_array(arr).ok_or(DeserializationError::ConstructionFailure)
    }
}
