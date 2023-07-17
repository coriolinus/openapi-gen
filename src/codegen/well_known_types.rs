use super::Scalar;

/// These external types are exceptions to the general rule that a schema must be entirely self-contained.
pub const WELL_KNOWN_TYPES: &[(&str, Scalar)] = &[
    #[cfg(feature = "api-problem")]
    (
        "https://opensource.zalando.com/restful-api-guidelines/models/problem-1.0.1.yaml#/Problem",
        Scalar::ApiProblem,
    ),
];

pub fn find_well_known_type(ty: &str) -> Option<Scalar> {
    // for a short list like we expect from WKTs, straight scan will be faster
    // than any kind of more complicated hashing scheme
    WELL_KNOWN_TYPES
        .iter()
        .find_map(|(t, s)| (*t == ty).then_some(*s))
}
