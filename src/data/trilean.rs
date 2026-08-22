//! Three-valued logic (`TRUE` / `FALSE` / `UNKNOWN`) for SQL predicate evaluation.
//!
//! SQL does not evaluate predicates in boolean logic — it evaluates them in
//! *three-valued* logic, where a comparison involving NULL yields `UNKNOWN`
//! rather than true or false. `UNKNOWN` is not "false"; the distinction only
//! collapses at the very end, where `WHERE` keeps a row **only if the predicate
//! is `TRUE`**. That deferred collapse is the whole point: `NOT UNKNOWN` is
//! `UNKNOWN` (still not kept), whereas `!false` is `true` (kept, wrongly).
//!
//! The WHERE evaluator historically returned `Result<bool>`, so `UNKNOWN` could
//! not be represented at all and `NOT` / `NOT IN` flipped it into `TRUE` —
//! producing extra rows. See findings P18/P19 in `docs/SQL_PARITY.md`.
//!
//! This module is deliberately standalone: it defines the truth tables and
//! proves them with tests, ahead of the evaluator being converted to use it.

use std::fmt;
use std::ops::{BitAnd, BitOr, Not};

/// A SQL truth value: `TRUE`, `FALSE`, or `UNKNOWN`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trilean {
    True,
    False,
    /// The result of any comparison involving NULL.
    Unknown,
}

impl Trilean {
    /// Lift a boolean into three-valued logic.
    pub fn from_bool(b: bool) -> Self {
        if b {
            Trilean::True
        } else {
            Trilean::False
        }
    }

    /// Lift a comparison whose operands may be NULL: `None` means an operand
    /// was NULL, which is `UNKNOWN` — never `FALSE`.
    pub fn from_option(b: Option<bool>) -> Self {
        match b {
            Some(v) => Trilean::from_bool(v),
            None => Trilean::Unknown,
        }
    }

    /// The single semantic boundary: `WHERE` / `HAVING` / `ON` keep a row only
    /// when the predicate is `TRUE`. `UNKNOWN` is dropped, exactly like `FALSE`.
    ///
    /// This is the *only* sanctioned way to turn a `Trilean` back into a
    /// `bool` for row filtering — collapsing earlier is what reintroduces
    /// P18/P19.
    pub fn is_true(self) -> bool {
        matches!(self, Trilean::True)
    }

    pub fn is_false(self) -> bool {
        matches!(self, Trilean::False)
    }

    pub fn is_unknown(self) -> bool {
        matches!(self, Trilean::Unknown)
    }

    /// SQL `AND`. `UNKNOWN AND FALSE` is `FALSE` — a false operand short-circuits
    /// even when the other side is unknown.
    pub fn and(self, other: Trilean) -> Trilean {
        match (self, other) {
            (Trilean::False, _) | (_, Trilean::False) => Trilean::False,
            (Trilean::True, Trilean::True) => Trilean::True,
            _ => Trilean::Unknown,
        }
    }

    /// SQL `OR`. `UNKNOWN OR TRUE` is `TRUE` — a true operand short-circuits
    /// even when the other side is unknown.
    pub fn or(self, other: Trilean) -> Trilean {
        match (self, other) {
            (Trilean::True, _) | (_, Trilean::True) => Trilean::True,
            (Trilean::False, Trilean::False) => Trilean::False,
            _ => Trilean::Unknown,
        }
    }

    /// SQL `NOT`. `NOT UNKNOWN` is `UNKNOWN`, **not** `TRUE`.
    pub fn negate(self) -> Trilean {
        match self {
            Trilean::True => Trilean::False,
            Trilean::False => Trilean::True,
            Trilean::Unknown => Trilean::Unknown,
        }
    }

    /// `IS TRUE` / `IS FALSE` / `IS UNKNOWN` predicates, which are themselves
    /// always two-valued: they never yield `UNKNOWN`.
    pub fn is_predicate(self, expected: Trilean) -> Trilean {
        Trilean::from_bool(self == expected)
    }
}

impl From<bool> for Trilean {
    fn from(b: bool) -> Self {
        Trilean::from_bool(b)
    }
}

impl From<Option<bool>> for Trilean {
    fn from(b: Option<bool>) -> Self {
        Trilean::from_option(b)
    }
}

impl BitAnd for Trilean {
    type Output = Trilean;
    fn bitand(self, rhs: Trilean) -> Trilean {
        self.and(rhs)
    }
}

impl BitOr for Trilean {
    type Output = Trilean;
    fn bitor(self, rhs: Trilean) -> Trilean {
        self.or(rhs)
    }
}

impl Not for Trilean {
    type Output = Trilean;
    fn not(self) -> Trilean {
        self.negate()
    }
}

impl fmt::Display for Trilean {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Trilean::True => "TRUE",
            Trilean::False => "FALSE",
            Trilean::Unknown => "UNKNOWN",
        };
        f.write_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::Trilean::{False, True, Unknown};
    use super::*;

    const ALL: [Trilean; 3] = [True, False, Unknown];

    // --- Truth tables, written out in full rather than derived, so a change to
    // --- the implementation cannot silently change what we assert.

    #[test]
    fn and_truth_table() {
        let table = [
            (True, True, True),
            (True, False, False),
            (True, Unknown, Unknown),
            (False, True, False),
            (False, False, False),
            (False, Unknown, False), // FALSE dominates, even over UNKNOWN
            (Unknown, True, Unknown),
            (Unknown, False, False), // ditto, reversed
            (Unknown, Unknown, Unknown),
        ];
        for (a, b, expected) in table {
            assert_eq!(a.and(b), expected, "{a} AND {b}");
            assert_eq!(a & b, expected, "{a} & {b}");
        }
    }

    #[test]
    fn or_truth_table() {
        let table = [
            (True, True, True),
            (True, False, True),
            (True, Unknown, True), // TRUE dominates, even over UNKNOWN
            (False, True, True),
            (False, False, False),
            (False, Unknown, Unknown),
            (Unknown, True, True), // ditto, reversed
            (Unknown, False, Unknown),
            (Unknown, Unknown, Unknown),
        ];
        for (a, b, expected) in table {
            assert_eq!(a.or(b), expected, "{a} OR {b}");
            assert_eq!(a | b, expected, "{a} | {b}");
        }
    }

    #[test]
    fn not_truth_table() {
        assert_eq!(True.negate(), False);
        assert_eq!(False.negate(), True);
        // The whole reason this type exists: NOT UNKNOWN is UNKNOWN, and a
        // bool-valued evaluator turns it into TRUE. See P18/P19.
        assert_eq!(Unknown.negate(), Unknown);
        for a in ALL {
            assert_eq!(!a, a.negate());
        }
    }

    // --- Algebraic laws. These hold in 3VL and are the properties the
    // --- evaluator's rewrites (De Morgan, double negation) rely on.

    #[test]
    fn double_negation_is_identity() {
        for a in ALL {
            assert_eq!(a.negate().negate(), a, "NOT NOT {a}");
        }
    }

    #[test]
    fn de_morgan_holds() {
        for a in ALL {
            for b in ALL {
                assert_eq!(
                    (a.and(b)).negate(),
                    a.negate().or(b.negate()),
                    "NOT({a} AND {b})"
                );
                assert_eq!(
                    (a.or(b)).negate(),
                    a.negate().and(b.negate()),
                    "NOT({a} OR {b})"
                );
            }
        }
    }

    #[test]
    fn and_or_are_commutative_and_associative() {
        for a in ALL {
            for b in ALL {
                assert_eq!(a.and(b), b.and(a));
                assert_eq!(a.or(b), b.or(a));
                for c in ALL {
                    assert_eq!(a.and(b).and(c), a.and(b.and(c)));
                    assert_eq!(a.or(b).or(c), a.or(b.or(c)));
                }
            }
        }
    }

    #[test]
    fn identity_and_annihilator_elements() {
        for a in ALL {
            assert_eq!(a.and(True), a, "{a} AND TRUE");
            assert_eq!(a.and(False), False, "{a} AND FALSE");
            assert_eq!(a.or(False), a, "{a} OR FALSE");
            assert_eq!(a.or(True), True, "{a} OR TRUE");
        }
    }

    /// The law of excluded middle does *not* hold in 3VL — `x OR NOT x` is
    /// UNKNOWN when x is UNKNOWN. Asserting this pins the difference from bool.
    #[test]
    fn excluded_middle_fails_for_unknown() {
        assert_eq!(True.or(True.negate()), True);
        assert_eq!(False.or(False.negate()), True);
        assert_eq!(Unknown.or(Unknown.negate()), Unknown);
        assert_eq!(Unknown.and(Unknown.negate()), Unknown);
    }

    // --- The boundary.

    #[test]
    fn where_keeps_only_true() {
        assert!(True.is_true());
        assert!(!False.is_true());
        // UNKNOWN rows are dropped by WHERE, same as FALSE.
        assert!(!Unknown.is_true());
    }

    #[test]
    fn boolean_subset_matches_two_valued_logic() {
        // Over TRUE/FALSE only, Trilean must behave exactly like bool, which is
        // what makes wiring it in a provable no-op until the NULL paths change.
        for a in [true, false] {
            for b in [true, false] {
                let (ta, tb) = (Trilean::from_bool(a), Trilean::from_bool(b));
                assert_eq!(ta.and(tb).is_true(), a && b);
                assert_eq!(ta.or(tb).is_true(), a || b);
                assert_eq!(ta.negate().is_true(), !a);
            }
        }
    }

    #[test]
    fn lifting_from_option() {
        assert_eq!(Trilean::from_option(Some(true)), True);
        assert_eq!(Trilean::from_option(Some(false)), False);
        // A NULL operand is UNKNOWN, never FALSE — the mistake this type exists
        // to prevent.
        assert_eq!(Trilean::from_option(None), Unknown);
        assert_eq!(Trilean::from(None::<bool>), Unknown);
        assert_eq!(Trilean::from(true), True);
    }

    #[test]
    fn is_predicates_are_two_valued() {
        for a in ALL {
            for expected in ALL {
                let r = a.is_predicate(expected);
                assert!(!r.is_unknown(), "{a} IS {expected} must not be UNKNOWN");
                assert_eq!(r.is_true(), a == expected);
            }
        }
    }

    #[test]
    fn display_uses_sql_spelling() {
        assert_eq!(True.to_string(), "TRUE");
        assert_eq!(False.to_string(), "FALSE");
        assert_eq!(Unknown.to_string(), "UNKNOWN");
    }
}
