//! This module contains hashing sequences with included domain separation tags
//! shared between different parts of the code.

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::curve::{CurvePoint, CurveScalar};
use crate::hashing::ScalarDigest;
use crate::key_frag::KeyFragID;
use crate::keys::PublicKey;
use crate::traits::SerializableToArray;

// TODO (#39): Ideally this should return a non-zero scalar.
pub(crate) fn hash_to_polynomial_arg(
    precursor: &CurvePoint,
    pubkey: &CurvePoint,
    dh_point: &CurvePoint,
    kfrag_id: &KeyFragID,
) -> CurveScalar {
    ScalarDigest::new_with_dst(b"POLYNOMIAL_ARG")
        .chain_point(precursor)
        .chain_point(pubkey)
        .chain_point(dh_point)
        .chain_bytes(kfrag_id)
        .finalize()
}

pub(crate) fn hash_to_shared_secret(
    precursor: &CurvePoint,
    pubkey: &CurvePoint,
    dh_point: &CurvePoint,
) -> CurveScalar {
    ScalarDigest::new_with_dst(b"SHARED_SECRET")
        .chain_point(precursor)
        .chain_point(pubkey)
        .chain_point(dh_point)
        .finalize()
}

pub(crate) fn hash_capsule_points(capsule_e: &CurvePoint, capsule_v: &CurvePoint) -> CurveScalar {
    ScalarDigest::new_with_dst(b"CAPSULE_POINTS")
        .chain_point(capsule_e)
        .chain_point(capsule_v)
        .finalize()
}

pub(crate) fn hash_to_cfrag_verification(
    points: &[CurvePoint],
    metadata: Option<&[u8]>,
) -> CurveScalar {
    let digest = ScalarDigest::new_with_dst(b"CFRAG_VERIFICATION").chain_points(points);

    let digest = match metadata {
        Some(s) => digest.chain_bytes(s),
        None => digest,
    };

    digest.finalize()
}

pub(crate) fn kfrag_signature_message(
    kfrag_id: &KeyFragID,
    commitment: &CurvePoint,
    precursor: &CurvePoint,
    maybe_delegating_pk: Option<&PublicKey>,
    maybe_receiving_pk: Option<&PublicKey>,
) -> Box<[u8]> {
    let mut result = Vec::<u8>::new();

    result.extend_from_slice(&kfrag_id.to_array());
    result.extend_from_slice(&commitment.to_array());
    result.extend_from_slice(&precursor.to_array());

    match maybe_delegating_pk {
        Some(delegating_pk) => {
            result.extend_from_slice(&true.to_array());
            result.extend_from_slice(&delegating_pk.to_array())
        }
        None => result.extend_from_slice(&false.to_array()),
    };

    match maybe_receiving_pk {
        Some(receiving_pk) => {
            result.extend_from_slice(&true.to_array());
            result.extend_from_slice(&receiving_pk.to_array())
        }
        None => result.extend_from_slice(&false.to_array()),
    };

    result.into_boxed_slice()
}
